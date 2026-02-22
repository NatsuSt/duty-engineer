use asterisk_manager::{Manager, AmiAction};
use log::{debug, info, trace};
use tauri::{AppHandle};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use std::collections::HashMap;
use crate::config::{AmiOperator, AppConfig};
use anyhow::Result;


/// Initializes and authenticates an Asterisk AMI session.
///
/// ### Arguments
/// * `app_handle` - A reference to the application handle for UI interactions.
/// * `config` - Application configuration containing AMI credentials.
///
/// ### Returns
/// * `Some(Manager)` - A logged-in manager instance ready for command processing.
/// * `None` - If the AMI configuration is not found.
///
/// ### Note
/// If the connection attempt fails, a modal dialog will be shown to the user.
pub async fn register_client(app_handle: &AppHandle, config: &AppConfig) -> Option<Manager> {
    let options = config.ami_manager.clone()?;
    let mut manager = Manager::new();

    // Establishes connection to Asterisk and handles the result
    let result = manager.connect_and_login(options).await;
    if let Err(e) = result {
        trace!("Error occured: {:#?}", e);
        app_handle.dialog()
            .message("Не знайдено Asterisk")
            .kind(MessageDialogKind::Info)
            .title("Попередження")
            .buttons(MessageDialogButtons::Ok)
            .show(|_| {});
    }
    info!("Successfully connected to AMI!");

    Some(manager)
}



/// Initiates a call to an engineer via the Asterisk Manager Interface (AMI).
///
/// # Preconditions
/// This function should only be called after verifying that a connection 
/// to the Asterisk server has been established.
///
/// # Arguments
/// * `manager` - A reference to the active AMI manager.
/// * `phone_number` - The destination phone number to dial.
/// * `ami_operator` - Configuration containing the operator's channel and context.
pub async fn make_call(manager: &Manager, phone_number: &str, ami_operator: &AmiOperator) -> Result<(), String>{
    debug!("Make_call function is called");
    // Standardize phone number format before sending
    let phone_number = parse_phone(phone_number);

    // Prepare the Originate action parameters
    let mut params = HashMap::new();
    params.insert("Channel".into(), format!("SIP/{}", ami_operator.operator_number));
    params.insert("Context".into(), ami_operator.context.clone());
    params.insert("Exten".into(), phone_number);
    params.insert("Priority".into(), "1".into());
    params.insert("Async".into(), "true".into());
    params.insert("CallerID".into(), "Дежурный оператор".into());

    let action = AmiAction::Custom {
        action: "Originate".into(),
        params,
        action_id: None,
    };

    // Send the action and transform any network/AMI errors into a String Result
    let response = manager.send_action(action).await
        .map_err(|e| format!("Failed to send AMI action: {}", e))?;
    trace!("Phone call response: {:#?}", response);
    
    Ok(())
}


/// Standardize phone number for using with Asterisk
/// 
/// Convert only phone numbers for ukrainian region
/// From `+38(050)-111-11-11` to 0501111111
/// # Argument
/// * `phone_number` - Phone number to parse
fn parse_phone(phone_number: &str) -> String {
    phone_number.replace("(", "")
        .replace(")", "")
        .replace("-", "")
        .replace("+38", "")
}
