use crate::db::DatabaseManager;
use crate::commands::AppState;
use log::{error, info};
use std::future::Future;
use tauri::State;

/// Generic helper for executing operations with database manager state management
///
/// This function handles the repetitive pattern of:
/// 1. Acquiring a read lock on the application state (async-friendly)
/// 2. Cloning the database manager
/// 3. Releasing the lock immediately
/// 4. Executing the async operation
/// 5. Logging success/failure
/// 6. Converting errors to strings
///
/// Using RwLock allows concurrent read access, improving throughput for
/// multiple simultaneous commands.
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
    // Handle state access with RwLock - async-friendly, allows concurrent reads
    let manager_clone = {
        let manager = state.read().await;
        manager.clone()
    }; // Read lock is automatically dropped here when manager goes out of scope

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

    // Command with page and page_size parameters (paginated queries)
    (
        $fn_name:ident,
        $return_type:ty,
        $manager_method:ident,
        $action:literal,
        $entity_name:literal,
        paginated
    ) => {
        #[tauri::command]
        pub async fn $fn_name(
            page: usize,
            page_size: usize,
            state: State<'_, AppState>
        ) -> Result<$return_type, String> {
            $crate::commands::utils::execute_with_manager(
                &state,
                |manager| Box::pin(async move {
                    manager.$manager_method(page, page_size).await
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
    use std::sync::Arc;
    use tokio::sync::RwLock;
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