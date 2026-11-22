"""Native MSI authoring, Python 3.9/3.10 on Windows. Does not install anything."""
import pathlib, sys, uuid, tempfile, subprocess, hashlib
import msilib
from msilib import schema, sequence
UPGRADE = '{0DA4CC95-474C-4DB7-A9C0-183BB836DDB2}'
def guid(key): return '{' + str(uuid.uuid5(uuid.UUID(UPGRADE.strip('{}')), key)).upper() + '}'
def build(exe, output, version):
    exe, output = pathlib.Path(exe).resolve(), pathlib.Path(output).resolve()
    assert exe.is_file(), 'Build the exe first'
    output.parent.mkdir(parents=True, exist_ok=True)
    db = msilib.init_database(str(output), schema, 'Orbit Axion', guid('product:'+version), version, 'Orbit Axion')
    si = db.GetSummaryInformation(20)
    si.SetProperty(msilib.PID_TEMPLATE, 'x64;1033')
    si.SetProperty(msilib.PID_PAGECOUNT, 500)
    si.SetProperty(msilib.PID_WORDCOUNT, 2)
    si.Persist()
    msilib.add_tables(db, sequence)
    def data(table, rows):
        for row in rows:
            if table.endswith('Sequence'):
                view=db.OpenView("SELECT Action FROM `%s` WHERE Action='%s'"%(table,row[0]));view.Execute(None)
                exists=view.Fetch();view.Close()
                if exists:
                    condition='NULL' if row[1] is None else "'%s'"%row[1].replace("'","''")
                    db.OpenView("UPDATE `%s` SET Condition=%s, Sequence=%s WHERE Action='%s'"%(table,condition,row[2],row[0])).Execute(None)
                    continue
            msilib.add_data(db, table, [row])
    data('Property', [('UpgradeCode',UPGRADE),('ALLUSERS','2'),('MSIINSTALLPERUSER','1'),('INSTALLLEVEL','1'),('ARPNOMODIFY','1'),('SecureCustomProperties','ALLUSERS;MSIINSTALLPERUSER;OLDPRODUCTS;NEWERPRODUCTS'),('DefaultUIFont','OrbitFont')])
    data('TextStyle',[('OrbitFont','Segoe UI',10,None,0),('OrbitTitle','Segoe UI',18,0x00ACAC,1)])
    data('LaunchCondition',[('VersionNT64','Orbit Axion requires Windows x64.'),('NOT NEWERPRODUCTS','A newer Orbit Axion version is installed.')])
    data('AppSearch',[('WINDOWSBUILD','WindowsBuild')])
    data('RegLocator',[('WindowsBuild',2,r'SOFTWARE\Microsoft\Windows NT\CurrentVersion','CurrentBuildNumber',18)])
    data('InstallUISequence',[('FindRelatedProducts',None,25),('AppSearch',None,50),('LaunchConditions',None,100)])
    data('InstallExecuteSequence',[('FindRelatedProducts',None,25),('AppSearch',None,50),('LaunchConditions',None,100)])
    data('LaunchCondition',[('WINDOWSBUILD >= 22000','Orbit Axion requires Windows 11 or newer.')])
    data('Upgrade',[(UPGRADE,None,version,None,1,None,'OLDPRODUCTS'),(UPGRADE,version,None,None,258,None,'NEWERPRODUCTS')])
    db.OpenView("UPDATE InstallExecuteSequence SET Sequence=1550 WHERE Action='RemoveExistingProducts'").Execute(None)
    cab = msilib.CAB('OrbitCab')
    root = msilib.Directory(db,cab,None,str(exe.parent),'TARGETDIR','SourceDir')
    programs = msilib.Directory(db,cab,root,'.','ProgramFiles64Folder','.')
    app = msilib.Directory(db,cab,programs,'.','INSTALLDIR','OrbitAxion')
    menu = msilib.Directory(db,cab,root,'.','ProgramMenuFolder','.')
    feature = msilib.Feature(db,'Main','Orbit Axion','Application, editor and tray',1,directory='INSTALLDIR')
    app.start_component('OrbitExecutable',feature,256,keyfile=exe.name,uuid=guid('exe'))
    fileid = app.add_file(exe.name)
    app.start_component('OrbitLicense',feature,256,uuid=guid('license'))
    app.add_file('LICENSE.txt',src=str(pathlib.Path(__file__).resolve().parents[2]/'LICENSE'))
    notices = pathlib.Path(__file__).resolve().parents[2]/'dist'/'THIRD_PARTY_NOTICES.txt'
    if not notices.is_file(): raise RuntimeError('Run collect_notices.py first')
    app.start_component('OrbitNotices',feature,256,uuid=guid('notices'))
    app.add_file('NOTICES.txt',src=str(notices))
    menu.start_component('OrbitShortcut',feature,256,uuid=guid('shortcut'))
    data('Registry',[('OrbitInstall',-1,r'Software\OrbitAxion','InstallPath','[INSTALLDIR]','OrbitShortcut')])
    db.OpenView("UPDATE Component SET KeyPath='OrbitInstall', Attributes=260 WHERE Component='OrbitShortcut'").Execute(None)
    data('Shortcut',[('OrbitStart','ProgramMenuFolder','Orbit|Orbit Axion','OrbitShortcut','[#%s]'%fileid,'--watchdog','Open Orbit Axion',None,None,None,1,'INSTALLDIR')])
    welcome=msilib.Dialog(db,'OrbitWelcome',50,50,370,245,3,'Orbit Axion','Install','Install','Cancel')
    welcome.control('Title','Text',20,18,330,35,3,None,'{\\OrbitTitle}Orbit Axion',None,None)
    welcome.control('Text','Text',20,60,325,45,3,None,'Radial actions for Windows. GNU GPL v3. Choose scope:',None,None)
    user=welcome.control('User','PushButton',20,115,155,24,3,None,'Only for me','Machine',None)
    user.event('[MSIINSTALLPERUSER]','1','NOT Installed',1)
    machine=welcome.control('Machine','PushButton',190,115,155,24,3,None,'Everyone (requires UAC)','Install',None)
    machine.event('[MSIINSTALLPERUSER]','{}','NOT Installed',1)
    welcome.control('Scope','Text',20,150,325,25,3,None,'Per-user: [MSIINSTALLPERUSER] (1 = only me)',None,None)
    install=welcome.control('Install','PushButton',190,205,75,24,3,None,'Install','Cancel',None)
    install.event('EndDialog','Return','1',1)
    cancel=welcome.control('Cancel','PushButton',275,205,75,24,3,None,'Cancel','User',None)
    cancel.event('EndDialog','Exit','1',1)
    data('InstallUISequence',[('OrbitWelcome','NOT Installed',750)])
    for name,title,text in [('ExitDialog','Finished','Operation completed. Open Orbit Axion from the Start menu.'),('FatalError','Failed','Windows Installer failed. The installation transaction is rolled back.'),('UserExit','Cancelled','Operation cancelled.')]:
        dlg=msilib.Dialog(db,name,50,50,370,200,3,title,'Close','Close','Close')
        dlg.control('Text','Text',20,25,330,100,3,None,text,None,None)
        dlg.control('Close','PushButton',275,155,75,24,3,None,'Close',None,None).event('EndDialog','Return','1',1)
    data('CustomAction',[('OrbitCleanup',18,fileid,'--uninstall-settings')])
    data('InstallUISequence',[('OrbitCleanup','Installed AND REMOVE="ALL" AND UILevel >= 3',1250)])
    with tempfile.TemporaryDirectory(prefix="orbit-msi-",dir=str(output.parent)) as scratch:
        scratch=pathlib.Path(scratch).resolve()
        if scratch.parent!=output.parent: raise RuntimeError("Unexpected scratch path")
        archive=scratch/'payload.cab'
        msilib.FCICreate(str(archive),cab.files)
        data('Media',[(1,cab.index,None,'#'+cab.name,None,None)])
        msilib.add_stream(db,cab.name,str(archive))
        extracted=scratch/'extracted';extracted.mkdir()
        subprocess.run(['expand.exe',str(archive),'-F:*',str(extracted)],check=True,capture_output=True)
        for source, name in cab.files:
            if hashlib.sha256(pathlib.Path(source).read_bytes()).digest()!=hashlib.sha256((extracted/name).read_bytes()).digest():
                raise RuntimeError('Payload checksum mismatch: '+name)

    db.Commit()
    print(output)
if __name__=='__main__': build(*sys.argv[1:])
