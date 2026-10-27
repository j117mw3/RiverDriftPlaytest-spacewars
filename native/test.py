from pathlib import Path
import os, shutil, subprocess, tempfile

root = Path(__file__).resolve().parent
build = root / 'build'
csc = Path(os.environ['WINDIR']) / 'Microsoft.NET/Framework64/v4.0.30319/csc.exe'
if not csc.is_file():
    raise SystemExit('The .NET Framework 4 x64 C# compiler is required.')
for filename in ('winmm.dll', 'winmm-test.dll', 'system-exports.json'):
    if not (build / filename).is_file():
        raise SystemExit('Run python native/build.py first: missing ' + filename)
with tempfile.TemporaryDirectory(prefix='spacewar-qa-', dir=build) as tmp:
    tmp = Path(tmp)
    exe = tmp / 'NativeProbe.exe'
    subprocess.run([str(csc), '/nologo', '/platform:x64', '/target:exe',
                    '/reference:System.Web.Extensions.dll', '/out:' + str(exe),
                    str(root / 'NativeProbe.cs')], check=True)
    cases = [
        ('game-480', 'RiverDrift-Win64-Shipping.exe', b'[Steam]\r\nAppId=480\r\n', '480'),
        ('default-id', 'RiverDrift-Win64-Shipping.exe', None, '480'),
        ('custom-id', 'RiverDrift-Win64-Shipping.exe', b'[Steam]\r\nAppId=481\r\n', '481'),
        ('invalid-id', 'RiverDrift-Win64-Shipping.exe', b'[Steam]\r\nAppId=bad\r\n', '480'),
        ('non-game-guard', 'OtherProbe.exe', b'[Steam]\r\nAppId=480\r\n', '4628450'),
    ]
    for name, executable, config, expected in cases:
        case = tmp / name
        case.mkdir()
        shutil.copy2(exe, case / executable)
        for filename in ('winmm.dll', 'winmm-test.dll', 'system-exports.json'):
            shutil.copy2(build / filename, case / filename)
        if config is not None:
            (case / 'RiverDrift-Spacewar.ini').write_bytes(config)
        subprocess.run([str(case / executable), str(case), expected], check=True)
print('All isolated native probe scenarios passed.')
