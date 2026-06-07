use windows::core::{Interface, BSTR, VARIANT};
use windows::Win32::Foundation::VARIANT_BOOL;
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED};
use windows::Win32::System::TaskScheduler::{
    IActionCollection, IExecAction, ILogonTrigger, IPrincipal, IRegisteredTask, ITaskDefinition,
    ITaskFolder, ITaskService, ITaskSettings, ITrigger, ITriggerCollection, TaskScheduler,
    TASK_ACTION_EXEC, TASK_CREATE_OR_UPDATE, TASK_INSTANCES_IGNORE_NEW,
    TASK_LOGON_INTERACTIVE_TOKEN, TASK_RUNLEVEL_HIGHEST, TASK_TRIGGER_LOGON,
};

use crate::error::UwdError;

const TASK_NAME: &str = "UWD2 Watermark Patch";

unsafe fn connect_service() -> Result<(ITaskService, ITaskFolder), UwdError> {
    let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

    let service: ITaskService =
        CoCreateInstance(&TaskScheduler, None, CLSCTX_ALL).map_err(UwdError::TaskCreateFailed)?;

    service
        .Connect(
            VARIANT::default(),
            VARIANT::default(),
            VARIANT::default(),
            VARIANT::default(),
        )
        .map_err(UwdError::TaskCreateFailed)?;

    let folder: ITaskFolder = service
        .GetFolder(&BSTR::from("\\"))
        .map_err(UwdError::TaskCreateFailed)?;

    Ok((service, folder))
}

pub fn install(exe_path: &str) -> Result<(), UwdError> {
    unsafe {
        let (service, folder) = connect_service()?;

        let task: ITaskDefinition = service.NewTask(0).map_err(UwdError::TaskCreateFailed)?;

        let reg_info = task.RegistrationInfo().map_err(UwdError::TaskCreateFailed)?;
        reg_info
            .SetDescription(&BSTR::from("Removes Windows Insider watermark at logon"))
            .map_err(UwdError::TaskCreateFailed)?;

        let principal: IPrincipal = task.Principal().map_err(UwdError::TaskCreateFailed)?;
        principal
            .SetLogonType(TASK_LOGON_INTERACTIVE_TOKEN)
            .map_err(UwdError::TaskCreateFailed)?;
        principal
            .SetRunLevel(TASK_RUNLEVEL_HIGHEST)
            .map_err(UwdError::TaskCreateFailed)?;

        let settings: ITaskSettings = task.Settings().map_err(UwdError::TaskCreateFailed)?;
        settings
            .SetMultipleInstances(TASK_INSTANCES_IGNORE_NEW)
            .map_err(UwdError::TaskCreateFailed)?;
        settings
            .SetStopIfGoingOnBatteries(VARIANT_BOOL(0))
            .map_err(UwdError::TaskCreateFailed)?;
        settings
            .SetDisallowStartIfOnBatteries(VARIANT_BOOL(0))
            .map_err(UwdError::TaskCreateFailed)?;
        settings
            .SetExecutionTimeLimit(&BSTR::from("PT2M"))
            .map_err(UwdError::TaskCreateFailed)?;

        let triggers: ITriggerCollection =
            task.Triggers().map_err(UwdError::TaskCreateFailed)?;
        let trigger: ITrigger = triggers
            .Create(TASK_TRIGGER_LOGON)
            .map_err(UwdError::TaskCreateFailed)?;
        let logon_trigger: ILogonTrigger =
            trigger.cast().map_err(UwdError::TaskCreateFailed)?;
        logon_trigger
            .SetEnabled(VARIANT_BOOL(-1))
            .map_err(UwdError::TaskCreateFailed)?;

        let actions: IActionCollection = task.Actions().map_err(UwdError::TaskCreateFailed)?;
        let action = actions
            .Create(TASK_ACTION_EXEC)
            .map_err(UwdError::TaskCreateFailed)?;
        let exec_action: IExecAction = action.cast().map_err(UwdError::TaskCreateFailed)?;
        exec_action
            .SetPath(&BSTR::from(exe_path))
            .map_err(UwdError::TaskCreateFailed)?;

        folder
            .RegisterTaskDefinition(
                &BSTR::from(TASK_NAME),
                &task,
                TASK_CREATE_OR_UPDATE.0,
                VARIANT::default(),
                VARIANT::default(),
                TASK_LOGON_INTERACTIVE_TOKEN,
                VARIANT::default(),
            )
            .map_err(UwdError::TaskCreateFailed)?;

        println!("Scheduled task \"{TASK_NAME}\" created.");
        Ok(())
    }
}

pub fn uninstall() -> Result<(), UwdError> {
    unsafe {
        let result = connect_service();
        let (_, folder) = match result {
            Ok(v) => v,
            Err(UwdError::TaskCreateFailed(e)) => return Err(UwdError::TaskDeleteFailed(e)),
            Err(e) => return Err(e),
        };

        match folder.DeleteTask(&BSTR::from(TASK_NAME), 0) {
            Ok(_) => println!("Scheduled task \"{TASK_NAME}\" removed."),
            Err(e) => {
                // 0x80070002 = HRESULT for ERROR_FILE_NOT_FOUND — task didn't exist
                if e.code().0 == -2147024894i32 {
                    println!("Scheduled task \"{TASK_NAME}\" was not installed.");
                } else {
                    return Err(UwdError::TaskDeleteFailed(e));
                }
            }
        }
        Ok(())
    }
}

pub fn status() {
    unsafe {
        let Ok((_, folder)) = connect_service() else {
            eprintln!("Error: could not connect to Task Scheduler.");
            return;
        };

        match folder.GetTask(&BSTR::from(TASK_NAME)) {
            Ok(task) => {
                let enabled = task.Enabled().unwrap_or(VARIANT_BOOL(0));
                let state = if enabled.0 != 0 { "enabled" } else { "disabled" };
                println!("UWD2 Watermark Patch: installed ({state})");
                if let Ok(path) = get_task_exe_path(&task) {
                    println!("  Executable: {path}");
                }
            }
            Err(_) => {
                println!("UWD2 Watermark Patch: not installed.");
                println!("Run `uwd2 install` to set up automatic startup.");
            }
        }
    }
}

unsafe fn get_task_exe_path(task: &IRegisteredTask) -> Result<String, windows::core::Error> {
    let def = task.Definition()?;
    let actions = def.Actions()?;
    let action = actions.get_Item(1)?;
    let exec: IExecAction = action.cast()?;
    Ok(exec.Path()?.to_string())
}
