import { chromium } from 'playwright';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Ensure screenshots directory exists
const screenshotsDir = path.join(__dirname, 'screenshots');
if (!fs.existsSync(screenshotsDir)) {
  fs.mkdirSync(screenshotsDir, { recursive: true });
}

async function takeScreenshots() {
  const browser = await chromium.launch();
  const context = await browser.newContext({
    viewport: { width: 1280, height: 800 }
  });
  const page = await context.newPage();

  try {
    // Navigate to the app
    console.log('Navigating to app...');
    await page.goto('http://localhost:1420', { waitUntil: 'networkidle' });
    
    // Take dashboard screenshot
    await page.screenshot({ 
      path: path.join(screenshotsDir, 'dashboard-baseline.png'),
      fullPage: true 
    });
    console.log('✅ Dashboard screenshot taken');

    // Navigate to Companies page
    await page.click('text=Companies');
    await page.waitForTimeout(1000);
    await page.screenshot({ 
      path: path.join(screenshotsDir, 'companies-baseline.png'),
      fullPage: true 
    });
    console.log('✅ Companies page screenshot taken');

    // Open Company modal by clicking "New Company"
    try {
      await page.click('text=New Company', { timeout: 5000 });
      await page.waitForTimeout(1000);
      await page.screenshot({ 
        path: path.join(screenshotsDir, 'company-modal-baseline.png'),
        fullPage: true 
      });
      console.log('✅ Company modal screenshot taken');
      
      // Close modal
      await page.press('Escape');
      await page.waitForTimeout(500);
    } catch (e) {
      console.log('ℹ️  Company modal not accessible:', e.message);
    }

    // Navigate to Contacts page
    await page.click('text=Contacts');
    await page.waitForTimeout(1000);
    await page.screenshot({ 
      path: path.join(screenshotsDir, 'contacts-baseline.png'),
      fullPage: true 
    });
    console.log('✅ Contacts page screenshot taken');

    // Try to open Contact modal
    try {
      await page.click('text=New Contact', { timeout: 5000 });
      await page.waitForTimeout(1000);
      await page.screenshot({ 
        path: path.join(screenshotsDir, 'contact-modal-baseline.png'),
        fullPage: true 
      });
      console.log('✅ Contact modal screenshot taken');
      
      // Close modal
      await page.press('Escape');
      await page.waitForTimeout(500);
    } catch (e) {
      console.log('ℹ️  Contact modal not accessible:', e.message);
    }

    // Navigate to Projects page
    await page.click('text=Projects');
    await page.waitForTimeout(1000);
    await page.screenshot({ 
      path: path.join(screenshotsDir, 'projects-baseline.png'),
      fullPage: true 
    });
    console.log('✅ Projects page screenshot taken');

    console.log('\n🎉 All baseline screenshots captured successfully!');
    console.log(`📁 Screenshots saved to: ${screenshotsDir}`);

  } catch (error) {
    console.error('❌ Error taking screenshots:', error);
  } finally {
    await browser.close();
  }
}

// Install playwright if needed and run
takeScreenshots().catch(console.error);