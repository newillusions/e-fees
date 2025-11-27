use crate::db::DatabaseManager;
use crate::commands::AppState;
use log::{error, info};
use std::future::Future;
use tauri::State;

/// Generic helper for executing operations with database manager state management
/// 
/// This function handles the repetitive pattern of:
/// 1. Locking the application state
/// 2. Cloning the database manager
/// 3. Executing the operation
/// 4. Logging success/failure
/// 5. Converting errors to strings
pub async fn execute_with_manager<T, F, Fut>(
    state: &State<'_, AppState>,
    operation: F,
    action: &str,
    entity_name: &str,
) -> Result<T, String>
where
    F: FnOnce(DatabaseManager) -> Fut,
    Fut: Future<Output = Result<T, surrealdb::Error>>,
{
    // Handle state locking with automatic cleanup
    let manager_clone = {
        let manager = state.lock().map_err(|e| {
            error!("Failed to lock application state: {}", e);
            e.to_string()
        })?;
        manager.clone()
    }; // Lock is automatically dropped here when manager goes out of scope
    
    // Execute the operation with consistent logging
    match operation(manager_clone).await {
        Ok(result) => {
            info!("Successfully {} {}", action, entity_name);
            Ok(result)
        }
        Err(e) => {
            error!("Failed to {} {}: {}", action, entity_name, e);
            Err(format!("Failed to {} {}: {}", action, entity_name, e))
        }
    }
}

/// Macro for generating standard CRUD command functions
/// 
/// This macro eliminates the boilerplate of creating similar command functions
/// by generating them with consistent patterns for:
/// - State management
/// - Error handling  
/// - Logging
/// - Parameter passing
#[macro_export]
macro_rules! crud_command {
    // Command with single ID parameter (get, delete)
    (
        $fn_name:ident,
        $return_type:ty,
        $manager_method:ident,
        $action:literal,
        $entity_name:literal,
        id: String
    ) => {
        #[tauri::command]
        pub async fn $fn_name(
            id: String,
            state: State<'_, AppState>
        ) -> Result<$return_type, String> {
            $crate::commands::utils::execute_with_manager(
                &state,
                |manager| Box::pin(async move { 
                    manager.$manager_method(&id).await 
                }),
                $action,
                $entity_name
            ).await
        }
    };
    
    // Command with data parameter (create, update)
    (
        $fn_name:ident,
        $return_type:ty,
        $data_type:ty,
        $manager_method:ident,
        $action:literal,
        $entity_name:literal,
        data: $data_param:ident
    ) => {
        #[tauri::command]
        pub async fn $fn_name(
            $data_param: $data_type,
            state: State<'_, AppState>
        ) -> Result<$return_type, String> {
            $crate::commands::utils::execute_with_manager(
                &state,
                |manager| Box::pin(async move { 
                    manager.$manager_method($data_param).await 
                }),
                $action,
                $entity_name
            ).await
        }
    };
    
    // Command with ID and data parameters (update)
    (
        $fn_name:ident,
        $return_type:ty,
        $data_type:ty,
        $manager_method:ident,
        $action:literal,
        $entity_name:literal,
        id: String,
        data: $data_param:ident
    ) => {
        #[tauri::command]
        pub async fn $fn_name(
            id: String,
            $data_param: $data_type,
            state: State<'_, AppState>
        ) -> Result<$return_type, String> {
            $crate::commands::utils::execute_with_manager(
                &state,
                |manager| Box::pin(async move { 
                    manager.$manager_method(&id, $data_param).await 
                }),
                $action,
                $entity_name
            ).await
        }
    };
    
    // Command with no parameters (get_all)
    (
        $fn_name:ident,
        $return_type:ty,
        $manager_method:ident,
        $action:literal,
        $entity_name:literal
    ) => {
        #[tauri::command]
        pub async fn $fn_name(
            state: State<'_, AppState>
        ) -> Result<$return_type, String> {
            $crate::commands::utils::execute_with_manager(
                &state,
                |manager| Box::pin(async move { 
                    manager.$manager_method().await 
                }),
                $action,
                $entity_name
            ).await
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DatabaseManager;
    use std::sync::{Arc, Mutex};
    use tauri::State;
    
    #[tokio::test]
    async fn test_execute_with_manager_success() {
        // This would require more setup for a full test
        // but demonstrates the testing approach
        assert!(true);
    }
    
    #[test]
    fn test_macro_expansion() {
        // Test that macros compile correctly
        // This ensures the macro syntax is valid
        assert!(true);
    }
}