import { chromium } from 'playwright';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const screenshotsDir = path.join(__dirname, 'screenshots');

async function testRefactoredModal() {
  const browser = await chromium.launch({ headless: false });
  const context = await browser.newContext({
    viewport: { width: 1280, height: 800 }
  });
  const page = await context.newPage();

  try {
    console.log('🚀 Testing refactored CompanyModal...');
    
    // Navigate to app and wait for loading
    await page.goto('http://localhost:1420', { waitUntil: 'networkidle' });
    await page.waitForTimeout(2000);
    
    // Take screenshot of current state
    await page.screenshot({ 
      path: path.join(screenshotsDir, 'refactored-app-state.png'),
      fullPage: true 
    });
    console.log('📸 App state captured');
    
    // Navigate to Companies page
    console.log('📂 Navigating to Companies page...');
    await page.click('text=Companies');
    await page.waitForTimeout(1500);
    
    // Take screenshot of Companies page with refactored components
    await page.screenshot({ 
      path: path.join(screenshotsDir, 'companies-refactored.png'),
      fullPage: true 
    });
    console.log('📸 Refactored Companies page captured');
    
    console.log('✅ Refactored modal testing completed!');
    
  } catch (error) {
    console.error('❌ Error during refactored modal testing:', error);
  } finally {
    await browser.close();
  }
}

testRefactoredModal().catch(console.error);