import { chromium } from 'playwright';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const screenshotsDir = path.join(__dirname, 'screenshots');

async function captureModalScreenshots() {
  const browser = await chromium.launch({ headless: false }); // Non-headless for debugging
  const context = await browser.newContext({
    viewport: { width: 1280, height: 800 }
  });
  const page = await context.newPage();

  try {
    console.log('🚀 Starting modal screenshot capture...');
    
    // Navigate to app and wait for loading
    await page.goto('http://localhost:1420', { waitUntil: 'networkidle' });
    await page.waitForTimeout(2000);
    
    // Navigate to Companies page
    console.log('📂 Navigating to Companies page...');
    await page.click('text=Companies');
    await page.waitForTimeout(1500);
    
    // Try to open Company modal
    console.log('🏢 Attempting to open Company modal...');
    try {
      // Look for various possible button texts/selectors
      const addButtons = [
        'text=Add new company',
        'text=New Company', 
        'button[title*="company"]',
        '[data-testid="add-company"]',
        'button:has-text("Add")',
        'button[class*="add"]'
      ];
      
      let modalOpened = false;
      for (const selector of addButtons) {
        try {
          await page.click(selector, { timeout: 2000 });
          modalOpened = true;
          console.log(`✅ Modal opened using selector: ${selector}`);
          break;
        } catch (e) {
          console.log(`❌ Selector "${selector}" not found`);
        }
      }
      
      if (modalOpened) {
        await page.waitForTimeout(1000);
        await page.screenshot({ 
          path: path.join(screenshotsDir, 'company-modal-baseline.png'),
          fullPage: true 
        });
        console.log('📸 Company modal screenshot captured');
        
        // Close modal with Escape
        await page.keyboard.press('Escape');
        await page.waitForTimeout(500);
      } else {
        console.log('⚠️  Could not open Company modal - taking page screenshot instead');
        await page.screenshot({ 
          path: path.join(screenshotsDir, 'companies-page-with-buttons.png'),
          fullPage: true 
        });
      }
      
    } catch (e) {
      console.log('❌ Error with Company modal:', e.message);
    }
    
    // Navigate to Contacts page
    console.log('👥 Navigating to Contacts page...');
    await page.click('text=Contacts');
    await page.waitForTimeout(1500);
    
    // Try to open Contact modal
    console.log('👤 Attempting to open Contact modal...');
    try {
      const contactButtons = [
        'text=Add new contact',
        'text=New Contact',
        'button[title*="contact"]',
        '[data-testid="add-contact"]',
        'button:has-text("Add")',
        'button[class*="add"]'
      ];
      
      let contactModalOpened = false;
      for (const selector of contactButtons) {
        try {
          await page.click(selector, { timeout: 2000 });
          contactModalOpened = true;
          console.log(`✅ Contact modal opened using selector: ${selector}`);
          break;
        } catch (e) {
          console.log(`❌ Contact selector "${selector}" not found`);
        }
      }
      
      if (contactModalOpened) {
        await page.waitForTimeout(1000);
        await page.screenshot({ 
          path: path.join(screenshotsDir, 'contact-modal-baseline.png'),
          fullPage: true 
        });
        console.log('📸 Contact modal screenshot captured');
        
        // Close modal
        await page.keyboard.press('Escape');
        await page.waitForTimeout(500);
      } else {
        console.log('⚠️  Could not open Contact modal - taking page screenshot instead');
        await page.screenshot({ 
          path: path.join(screenshotsDir, 'contacts-page-with-buttons.png'),
          fullPage: true 
        });
      }
      
    } catch (e) {
      console.log('❌ Error with Contact modal:', e.message);
    }
    
    // Take a final screenshot of current page
    await page.screenshot({ 
      path: path.join(screenshotsDir, 'final-state.png'),
      fullPage: true 
    });
    
    console.log('🎉 Modal capture completed!');
    
  } catch (error) {
    console.error('❌ Error during modal capture:', error);
  } finally {
    await browser.close();
  }
}

captureModalScreenshots().catch(console.error);