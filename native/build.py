from pathlib import Path
import hashlib,json,os,shutil,struct,subprocess

HERE=Path(__file__).resolve().parent
OUT=HERE/'build'
OUT.mkdir(exist_ok=True)
SYSTEM=Path(os.environ['WINDIR'])/'System32/winmm.dll'

def pe_exports(path):
    b=path.read_bytes();pe=struct.unpack_from('<I',b,60)[0];machine,ns=struct.unpack_from('<HH',b,pe+4);opt=pe+24;optsz=struct.unpack_from('<H',b,pe+20)[0]
    assert b[:2]==b'MZ' and machine==0x8664
    er,es=struct.unpack_from('<II',b,opt+112)
    sections=[]
    for i in range(ns):
        pos=opt+optsz+40*i;vs,va,rs,rp=struct.unpack_from('<IIII',b,pos+8);flags=struct.unpack_from('<I',b,pos+36)[0];sections.append((va,max(vs,rs),rp,flags))
    def offset(rva):
        for va,size,raw,flags in sections:
            if va<=rva<va+size:return raw+rva-va
        raise ValueError(hex(rva))
    e=offset(er);base,nfunc,nname,af,an,ao=struct.unpack_from('<IIIIII',b,e+16);names={}
    for i in range(nname):
        r=struct.unpack_from('<I',b,offset(an)+i*4)[0];p=offset(r);name=b[p:b.index(b'\0',p)].decode('ascii');ordinal=struct.unpack_from('<H',b,offset(ao)+i*2)[0];names[ordinal]=name
    exports=[]
    for i in range(nfunc):
        rva=struct.unpack_from('<I',b,offset(af)+i*4)[0]
        if not rva:continue
        forwarded=er<=rva<er+es
        executable=any(va<=rva<va+size and flags&0x20000000 for va,size,raw,flags in sections)
        assert forwarded or executable,('unexpected data export',names.get(i),base+i)
        exports.append({'name':names.get(i),'ordinal':base+i})
    return exports

def wide(s):return '['+','.join(str(x) for x in (ord(c) for c in s+'\0'))+']'

def generate(exports,test=False):
    constants='pub const EXPORT_COUNT:usize='+str(len(exports))+';\n'
    constants+='static EXPORT_NAMES:[&[u8];EXPORT_COUNT]=['+','.join('b"'+(x['name'] or '')+'\\0"' for x in exports)+'];\n'
    constants+='static EXPORT_ORDINALS:[u16;EXPORT_COUNT]=['+','.join(str(x['ordinal']) for x in exports)+'];\n'
    for key,value in [('INI_FILE','RiverDrift-Spacewar.ini'),('LOG_FILE','RiverDrift-Spacewar-Loader.log'),('SYSTEM_SUFFIX','\\winmm.dll'),('STEAM_APP_ID','SteamAppId'),('STEAM_GAME_ID','SteamGameId'),('STEAM_OVERLAY_ID','SteamOverlayGameId')]:
        constants+='static '+key+':[u16;'+str(len(value)+1)+']='+wide(value)+';\n'
    (HERE/'generated.rs').write_text(constants,encoding='ascii')
    assembly=['.text'];definition=['LIBRARY winmm','EXPORTS']
    actual=list(exports)
    if test:actual += [{'name':'SW_TestIntegerAbi','ordinal':60000},{'name':'SW_TestFloatingAbi','ordinal':60001}]
    for index,entry in enumerate(actual):
        symbol='sw_export_'+str(index)
        definition.append('  '+(entry['name'] or ('Ordinal_'+str(entry['ordinal'])))+'='+symbol+' @'+str(entry['ordinal'])+(' NONAME' if entry['name'] is None else ''))
        assembly += ['.globl '+symbol,'.seh_proc '+symbol,symbol+':','sub rsp, 136','.seh_stackalloc 136','.seh_endprologue','mov [rsp+96], rcx','mov [rsp+104], rdx','mov [rsp+112], r8','mov [rsp+120], r9','movdqu [rsp+32], xmm0','movdqu [rsp+48], xmm1','movdqu [rsp+64], xmm2','movdqu [rsp+80], xmm3','mov ecx, '+str(index),'call sw_resolve_export','movdqu xmm0, [rsp+32]','movdqu xmm1, [rsp+48]','movdqu xmm2, [rsp+64]','movdqu xmm3, [rsp+80]','mov rcx, [rsp+96]','mov rdx, [rsp+104]','mov r8, [rsp+112]','mov r9, [rsp+120]','add rsp, 136','jmp rax','.seh_endproc']
    (HERE/'exports.s').write_text('\n'.join(assembly)+'\n',encoding='ascii')
    definition_path=OUT/('winmm-test.def' if test else 'winmm.def');definition_path.write_text('\n'.join(definition)+'\n',encoding='ascii')
    return definition_path

def build(test=False):
    exports=pe_exports(SYSTEM);definition=generate(exports,test)
    (OUT/'system-exports.json').write_text(json.dumps(exports,indent=2)+'\n',encoding='utf-8')
    sysroot=Path(subprocess.check_output(['rustc','--print','sysroot'],text=True).strip())
    linker=sysroot/'lib/rustlib/x86_64-pc-windows-msvc/bin/gcc-ld/lld-link.exe'
    archive=OUT/('spacewar-test.lib' if test else 'spacewar.lib');dll=OUT/('winmm-test.dll' if test else 'winmm.dll')
    cmd=['rustc','--edition=2021','--crate-type=staticlib','-C','panic=abort','-C','opt-level=2','-C','codegen-units=1','-C','debuginfo=0','--remap-path-prefix',str(HERE)+'=src',str(HERE/'spacewar_winmm.rs'),'-o',str(archive)]
    if test:cmd+=['--cfg','sw_test']
    subprocess.run(cmd,check=True,cwd=HERE)
    subprocess.run([str(linker),'/DLL','/NOLOGO','/NODEFAULTLIB','/ENTRY:DllMain','/MACHINE:X64','/OPT:REF','/OPT:ICF','/DYNAMICBASE','/NXCOMPAT','/timestamp:0','/DEF:'+str(definition),'/OUT:'+str(dll),str(archive)],check=True,cwd=HERE)
    built=pe_exports(dll);expected={(x['name'],x['ordinal']) for x in exports};assert expected.issubset({(x['name'],x['ordinal']) for x in built})
    print(json.dumps({'file':str(dll),'exports':len(built),'system_exports':len(exports),'bytes':dll.stat().st_size,'sha256':hashlib.sha256(dll.read_bytes()).hexdigest()}),flush=True)

if __name__=='__main__':
    build(False);build(True);generate(pe_exports(SYSTEM),False)
