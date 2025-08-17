// Simple test for folder management API
// This would be run inside the Tauri app context

async function testFolderManagement() {
  try {
    console.log('=== Testing Project Folder Management ===');
    
    // Test 1: Validate base path
    console.log('\n1. Testing base path validation...');
    const pathValidation = await invoke('validate_project_base_path');
    console.log('Path validation:', pathValidation);
    
    // Test 2: Find test project location
    console.log('\n2. Finding test project location...');
    const projectInfo = await invoke('get_project_folder_location', { 
      projectNumber: '25-97199' 
    });
    console.log('Project info:', projectInfo);
    
    // Test 3: List projects in RFP folder
    console.log('\n3. Listing projects in RFP folder...');
    const rfpProjects = await invoke('list_projects_in_folder', { 
      folderPath: '01 RFPs' 
    });
    console.log('RFP projects:', rfpProjects);
    
    // Test 4: Move project from RFP to Current (with template copying)
    if (projectInfo.exists && projectInfo.current_location === '01 RFPs') {
      console.log('\n4. Moving project from RFP to Current...');
      const moveResult = await invoke('move_project_from_rfp', { 
        projectNumber: '25-97199',
        destination: 'current'
      });
      console.log('Move result:', moveResult);
      
      // Test 5: Verify project is now in Current folder
      console.log('\n5. Verifying project moved to Current...');
      const updatedInfo = await invoke('get_project_folder_location', { 
        projectNumber: '25-97199' 
      });
      console.log('Updated project info:', updatedInfo);
      
      // Test 6: List projects in Current folder
      console.log('\n6. Listing projects in Current folder...');
      const currentProjects = await invoke('list_projects_in_folder', { 
        folderPath: '11 Current' 
      });
      console.log('Current projects:', currentProjects);
    }
    
    console.log('\n=== Folder Management Tests Complete ===');
    
  } catch (error) {
    console.error('Test failed:', error);
  }
}

// This function would be called from within the Tauri app
// testFolderManagement();