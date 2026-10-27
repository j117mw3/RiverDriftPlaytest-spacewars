using System;
using System.IO;
using System.Runtime.InteropServices;
using System.Threading;
using System.Web.Script.Serialization;

internal sealed class Export { public string name {get;set;} public int ordinal {get;set;} }
internal static class NativeProbe
{
    [DllImport("kernel32",CharSet=CharSet.Unicode,SetLastError=true)] private static extern IntPtr LoadLibraryW(string path);
    [DllImport("kernel32",CharSet=CharSet.Ansi)] private static extern IntPtr GetProcAddress(IntPtr module,string name);
    [DllImport("kernel32",EntryPoint="GetProcAddress")] private static extern IntPtr GetOrdinal(IntPtr module,IntPtr ordinal);
    [UnmanagedFunctionPointer(CallingConvention.Winapi)] private delegate uint Timer();
    [UnmanagedFunctionPointer(CallingConvention.Winapi)] private delegate uint Period(uint milliseconds);
    [StructLayout(LayoutKind.Sequential)] private struct Caps { public uint Minimum,Maximum; }
    [UnmanagedFunctionPointer(CallingConvention.Winapi)] private delegate uint GetCaps(out Caps caps,uint size);
    [UnmanagedFunctionPointer(CallingConvention.Winapi)] private delegate ulong IntegerAbi(ulong a,ulong b,ulong c,ulong d,ulong e,ulong f,ulong g,ulong h);
    [UnmanagedFunctionPointer(CallingConvention.Winapi)] private delegate double FloatAbi(double a,double b,double c,double d,double e,double f);
    private static T Function<T>(IntPtr module,string name) where T:class
    {
        IntPtr address=GetProcAddress(module,name);
        if(address==IntPtr.Zero) throw new Exception("Missing export: "+name);
        return Marshal.GetDelegateForFunctionPointer(address,typeof(T)) as T;
    }
    private static void Check(bool condition,string message) { if(!condition) throw new Exception(message); }
    private static int Main(string[] args)
    {
        try
        {
            string root=Path.GetFullPath(args[0]); string expected=args[1];
            foreach(string key in new[]{"SteamAppId","SteamGameId","SteamOverlayGameId"}) Environment.SetEnvironmentVariable(key,"4628450");
            IntPtr proxy=LoadLibraryW(Path.Combine(root,"winmm.dll"));
            Check(proxy!=IntPtr.Zero,"Native DLL failed to load: "+Marshal.GetLastWin32Error());
            foreach(string key in new[]{"SteamAppId","SteamGameId","SteamOverlayGameId"}) Check(Environment.GetEnvironmentVariable(key)==expected,"Unexpected "+key);
            Export[] exports=new JavaScriptSerializer().Deserialize<Export[]>(File.ReadAllText(Path.Combine(root,"system-exports.json")));
            foreach(Export item in exports)
            {
                Check(GetOrdinal(proxy,new IntPtr(item.ordinal))!=IntPtr.Zero,"Missing ordinal "+item.ordinal);
                if(item.name!=null) Check(GetProcAddress(proxy,item.name)!=IntPtr.Zero,"Missing name "+item.name);
            }
            Timer timer=Function<Timer>(proxy,"timeGetTime");
            Exception raceError=null; ManualResetEvent ready=new ManualResetEvent(false); Thread[] threads=new Thread[12];
            for(int i=0;i<threads.Length;i++)
            {
                threads[i]=new Thread(delegate(){try{ready.WaitOne();for(int j=0;j<100;j++) timer();}catch(Exception e){raceError=e;}});threads[i].Start();
            }
            ready.Set(); foreach(Thread thread in threads) thread.Join(); if(raceError!=null) throw raceError;
            IntPtr system=LoadLibraryW(Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.System),"winmm.dll"));
            Timer systemTimer=Function<Timer>(system,"timeGetTime"); uint before=systemTimer(),actual=timer(),after=systemTimer();
            Check(unchecked(actual-before)<=unchecked(after-before),"System timer forwarding differs");
            Period begin=Function<Period>(proxy,"timeBeginPeriod"), end=Function<Period>(proxy,"timeEndPeriod");
            Check(begin(1)==0,"timeBeginPeriod failed"); Check(end(1)==0,"timeEndPeriod failed");
            Check(Function<Timer>(proxy,"waveOutGetNumDevs")()==Function<Timer>(system,"waveOutGetNumDevs")(),"waveOutGetNumDevs mismatch");
            GetCaps capsCall=Function<GetCaps>(proxy,"timeGetDevCaps"); Caps caps;
            Check(capsCall(out caps,(uint)Marshal.SizeOf(typeof(Caps)))==0 && caps.Minimum>0 && caps.Maximum>=caps.Minimum,"Pointer/output forwarding failed");
            IntPtr test=LoadLibraryW(Path.Combine(root,"winmm-test.dll"));Check(test!=IntPtr.Zero,"ABI test DLL load failed");
            IntegerAbi ints=Function<IntegerAbi>(test,"SW_TestIntegerAbi");
            ulong integerExpected=1+2*2+3*3+4*4+5*5+6*0x100000006UL+7*7+8*8;
            Check(ints(1,2,3,4,5,0x100000006UL,7,8)==integerExpected,"Eight-argument integer/stack ABI failed");
            FloatAbi floats=Function<FloatAbi>(test,"SW_TestFloatingAbi");
            Check(Math.Abs(floats(1.25,2.5,3.75,5,6.25,7.5)-113.75)<0.000001,"SSE/stack floating-point ABI failed");
            Console.WriteLine("PASS "+Path.GetFileName(typeof(NativeProbe).Assembly.Location)+": expected identity "+expected+"; "+exports.Length+" export names/ordinals; concurrent initialization; native forwarding; eight-integer and six-float ABI.");
            return 0;
        }
        catch(Exception error) {Console.Error.WriteLine(error);return 1;}
    }
}
