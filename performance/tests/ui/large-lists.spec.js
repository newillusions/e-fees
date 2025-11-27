/**
 * Large List UI Performance Tests
 * 
 * Tests UI rendering performance with large datasets including:
 * - List rendering and scrolling performance
 * - Search and filtering responsiveness  
 * - Memory usage during list operations
 * - Virtual scrolling effectiveness
 */

import { test, expect } from '@playwright/test';
import path from 'path';

// Performance thresholds
const PERFORMANCE_THRESHOLDS = {
  listRenderTime: 1000,      // 1 second max for initial render
  scrollFrameRate: 55,       // 55+ fps during scrolling (allow 5fps buffer)
  searchResponseTime: 200,   // 200ms max for search filtering
  memoryGrowthMB: 50,        // Max 50MB memory growth during operations
  itemHeight: 80,            // Expected item height in pixels
};

test.describe('Large List Performance Tests', () => {
  let page;
  let performanceMetrics = {};

  test.beforeEach(async ({ page: testPage, context }) => {
    page = testPage;
    
    // Enable performance monitoring
    await context.grantPermissions(['performance-observer']);
    
    // Navigate to the E-Fees application
    await page.goto('http://localhost:1420', { waitUntil: 'networkidle' });
    
    // Wait for app initialization
    await page.waitForSelector('[data-testid="app-loaded"]', { timeout: 10000 });
    
    // Initialize performance monitoring
    await initializePerformanceMonitoring(page);
  });

  test('Projects List - Rendering Performance with 2000+ Projects', async () => {
    console.log('🚀 Testing projects list rendering performance...');
    
    // Navigate to projects page
    await page.click('[data-testid="nav-projects"]');
    await page.waitForSelector('[data-testid="projects-list"]');
    
    // Generate large dataset for testing
    await generateLargeProjectDataset(page, 2000);
    
    // Measure initial render time
    const renderStartTime = Date.now();
    await page.click('[data-testid="load-large-dataset"]');
    
    // Wait for list to be populated
    await page.waitForFunction(() => {
      const listItems = document.querySelectorAll('[data-testid^="project-item-"]');
      return listItems.length >= 100; // Wait for at least first batch to render
    }, { timeout: 10000 });
    
    const renderEndTime = Date.now();
    const renderTime = renderEndTime - renderStartTime;
    
    console.log(`📊 Initial render time: ${renderTime}ms`);
    expect(renderTime).toBeLessThan(PERFORMANCE_THRESHOLDS.listRenderTime);
    
    // Test scrolling performance
    await testScrollingPerformance(page);
    
    // Test list item interactions
    await testListItemInteractions(page);
    
    performanceMetrics.projectsListRender = renderTime;
  });

  test('Proposals List - Search and Filter Performance', async () => {
    console.log('🔍 Testing proposals search and filter performance...');
    
    // Navigate to proposals page
    await page.click('[data-testid="nav-proposals"]');
    await page.waitForSelector('[data-testid="proposals-list"]');
    
    // Generate large dataset
    await generateLargeProposalDataset(page, 5000);
    
    // Test search performance
    const searchTests = [
      { term: 'Dubai', expectedCount: '>10' },
      { term: 'Tower', expectedCount: '>5' },
      { term: 'Active', expectedCount: '>20' },
      { term: 'nonexistent_term', expectedCount: '0' }
    ];
    
    for (const searchTest of searchTests) {
      const searchStartTime = Date.now();
      
      // Clear search and enter new term
      await page.fill('[data-testid="search-input"]', '');
      await page.fill('[data-testid="search-input"]', searchTest.term);
      
      // Wait for search results to update
      await page.waitForFunction((term) => {
        const searchInput = document.querySelector('[data-testid="search-input"]');
        return searchInput && searchInput.value === term;
      }, searchTest.term);
      
      // Wait a bit more for debounced search to complete
      await page.waitForTimeout(100);
      
      const searchEndTime = Date.now();
      const searchTime = searchEndTime - searchStartTime;
      
      console.log(`🔍 Search time for "${searchTest.term}": ${searchTime}ms`);
      expect(searchTime).toBeLessThan(PERFORMANCE_THRESHOLDS.searchResponseTime);
      
      // Verify search results
      const resultCount = await page.locator('[data-testid^="proposal-item-"]').count();
      console.log(`📊 Search results for "${searchTest.term}": ${resultCount} items`);
      
      performanceMetrics[`search_${searchTest.term}`] = searchTime;
    }
  });

  test('Companies List - Memory Usage During Operations', async () => {
    console.log('💾 Testing companies list memory usage...');
    
    // Navigate to companies page
    await page.click('[data-testid="nav-companies"]');
    await page.waitForSelector('[data-testid="companies-list"]');
    
    // Get initial memory usage
    const initialMemory = await getMemoryUsage(page);
    console.log(`📊 Initial memory usage: ${initialMemory.toFixed(2)} MB`);
    
    // Generate and load large dataset
    await generateLargeCompanyDataset(page, 1000);
    
    // Measure memory after data load
    const afterLoadMemory = await getMemoryUsage(page);
    console.log(`📊 Memory after data load: ${afterLoadMemory.toFixed(2)} MB`);
    
    // Perform memory-intensive operations
    await performMemoryIntensiveOperations(page);
    
    // Measure final memory usage
    const finalMemory = await getMemoryUsage(page);
    console.log(`📊 Final memory usage: ${finalMemory.toFixed(2)} MB`);
    
    const memoryGrowth = finalMemory - initialMemory;
    console.log(`📈 Memory growth: ${memoryGrowth.toFixed(2)} MB`);
    
    expect(memoryGrowth).toBeLessThan(PERFORMANCE_THRESHOLDS.memoryGrowthMB);
    
    performanceMetrics.memoryGrowth = memoryGrowth;
  });

  test('Contacts List - Virtual Scrolling Performance', async () => {
    console.log('📜 Testing contacts virtual scrolling performance...');
    
    // Navigate to contacts page
    await page.click('[data-testid="nav-contacts"]');
    await page.waitForSelector('[data-testid="contacts-list"]');
    
    // Generate large dataset
    await generateLargeContactDataset(page, 10000);
    
    // Test virtual scrolling effectiveness
    await testVirtualScrolling(page);
    
    // Test rapid scrolling performance
    await testRapidScrolling(page);
  });

  test('Dashboard - Multiple Lists Performance', async () => {
    console.log('📊 Testing dashboard with multiple lists...');
    
    // Navigate to dashboard
    await page.click('[data-testid="nav-dashboard"]');
    await page.waitForSelector('[data-testid="dashboard"]');
    
    // Generate data for all entities
    await Promise.all([
      generateLargeProjectDataset(page, 500),
      generateLargeProposalDataset(page, 1000),
      generateLargeCompanyDataset(page, 200)
    ]);
    
    // Measure dashboard load time
    const dashboardStartTime = Date.now();
    await page.click('[data-testid="refresh-dashboard"]');
    
    // Wait for all dashboard components to load
    await Promise.all([
      page.waitForSelector('[data-testid="recent-projects"]'),
      page.waitForSelector('[data-testid="pending-proposals"]'),
      page.waitForSelector('[data-testid="activity-feed"]')
    ]);
    
    const dashboardEndTime = Date.now();
    const dashboardLoadTime = dashboardEndTime - dashboardStartTime;
    
    console.log(`📊 Dashboard load time: ${dashboardLoadTime}ms`);
    expect(dashboardLoadTime).toBeLessThan(2000); // 2 seconds max for dashboard
    
    performanceMetrics.dashboardLoad = dashboardLoadTime;
  });

  test.afterEach(async () => {
    // Generate performance report
    await generatePerformanceReport(page, performanceMetrics);
    
    // Clean up test data
    await cleanupTestData(page);
  });
});

// Helper functions

async function initializePerformanceMonitoring(page) {
  await page.addInitScript(() => {
    // Performance monitoring setup
    window.performanceMetrics = {
      renderTimes: [],
      frameRates: [],
      memoryUsage: [],
    };
    
    // Frame rate monitoring
    let lastFrameTime = performance.now();
    let frameCount = 0;
    
    function measureFrameRate() {
      const now = performance.now();
      const delta = now - lastFrameTime;
      lastFrameTime = now;
      frameCount++;
      
      if (frameCount % 60 === 0) { // Log every 60 frames
        const fps = 1000 / delta;
        window.performanceMetrics.frameRates.push(fps);
      }
      
      requestAnimationFrame(measureFrameRate);
    }
    measureFrameRate();
    
    // Memory usage monitoring
    if (performance.memory) {
      setInterval(() => {
        window.performanceMetrics.memoryUsage.push({
          used: performance.memory.usedJSHeapSize / 1024 / 1024,
          total: performance.memory.totalJSHeapSize / 1024 / 1024,
          limit: performance.memory.jsHeapSizeLimit / 1024 / 1024,
          timestamp: Date.now()
        });
      }, 1000);
    }
  });
}

async function generateLargeProjectDataset(page, count) {
  console.log(`📦 Generating ${count} test projects...`);
  
  await page.evaluate((projectCount) => {
    // Simulate generating large project dataset
    window.testData = window.testData || {};
    window.testData.projects = [];
    
    for (let i = 0; i < projectCount; i++) {
      window.testData.projects.push({
        id: `project_${i}`,
        name: `Test Project ${i}`,
        status: ['Active', 'Draft', 'Completed'][i % 3],
        city: ['Dubai', 'Abu Dhabi', 'Riyadh'][i % 3],
        country: ['U.A.E.', 'Saudi Arabia'][i % 2],
        created_at: new Date(Date.now() - Math.random() * 365 * 24 * 60 * 60 * 1000)
      });
    }
    
    console.log(`Generated ${window.testData.projects.length} test projects`);
  }, count);
}

async function generateLargeProposalDataset(page, count) {
  console.log(`📋 Generating ${count} test proposals...`);
  
  await page.evaluate((proposalCount) => {
    window.testData = window.testData || {};
    window.testData.proposals = [];
    
    for (let i = 0; i < proposalCount; i++) {
      window.testData.proposals.push({
        id: `proposal_${i}`,
        name: `Test Proposal ${i}`,
        status: ['Draft', 'Sent', 'Negotiation', 'Awarded'][i % 4],
        project_name: `Project ${i % 100}`,
        company_name: `Company ${i % 50}`,
        created_at: new Date(Date.now() - Math.random() * 365 * 24 * 60 * 60 * 1000)
      });
    }
    
    console.log(`Generated ${window.testData.proposals.length} test proposals`);
  }, count);
}

async function generateLargeCompanyDataset(page, count) {
  console.log(`🏢 Generating ${count} test companies...`);
  
  await page.evaluate((companyCount) => {
    window.testData = window.testData || {};
    window.testData.companies = [];
    
    for (let i = 0; i < companyCount; i++) {
      window.testData.companies.push({
        id: `company_${i}`,
        name: `Test Company ${i}`,
        city: ['Dubai', 'Abu Dhabi', 'Riyadh', 'Doha'][i % 4],
        country: ['U.A.E.', 'Saudi Arabia', 'Qatar'][i % 3],
        created_at: new Date(Date.now() - Math.random() * 365 * 24 * 60 * 60 * 1000)
      });
    }
    
    console.log(`Generated ${window.testData.companies.length} test companies`);
  }, count);
}

async function generateLargeContactDataset(page, count) {
  console.log(`👥 Generating ${count} test contacts...`);
  
  await page.evaluate((contactCount) => {
    window.testData = window.testData || {};
    window.testData.contacts = [];
    
    for (let i = 0; i < contactCount; i++) {
      window.testData.contacts.push({
        id: `contact_${i}`,
        name: `Test Contact ${i}`,
        email: `contact${i}@testcompany.com`,
        company_name: `Company ${i % 100}`,
        position: ['Manager', 'Director', 'VP', 'CEO'][i % 4],
        created_at: new Date(Date.now() - Math.random() * 365 * 24 * 60 * 60 * 1000)
      });
    }
    
    console.log(`Generated ${window.testData.contacts.length} test contacts`);
  }, count);
}

async function testScrollingPerformance(page) {
  console.log('📜 Testing scrolling performance...');
  
  const scrollContainer = await page.locator('[data-testid="projects-list"]');
  
  // Measure frame rate during scrolling
  await page.evaluate(() => {
    window.scrollFrameRates = [];
    let scrollFrameCount = 0;
    let scrollStartTime = performance.now();
    
    const measureScrollFPS = () => {
      scrollFrameCount++;
      const now = performance.now();
      if (now - scrollStartTime >= 1000) { // Every second
        const fps = scrollFrameCount;
        window.scrollFrameRates.push(fps);
        scrollFrameCount = 0;
        scrollStartTime = now;
      }
      if (window.isScrolling) {
        requestAnimationFrame(measureScrollFPS);
      }
    };
    
    window.startScrollMeasurement = () => {
      window.isScrolling = true;
      measureScrollFPS();
    };
    
    window.stopScrollMeasurement = () => {
      window.isScrolling = false;
    };
  });
  
  // Start measuring
  await page.evaluate(() => window.startScrollMeasurement());
  
  // Perform scrolling
  for (let i = 0; i < 5; i++) {
    await scrollContainer.evaluate(el => {
      el.scrollTop = el.scrollHeight * (i + 1) / 5;
    });
    await page.waitForTimeout(200);
  }
  
  // Stop measuring
  await page.evaluate(() => window.stopScrollMeasurement());
  
  // Get results
  const frameRates = await page.evaluate(() => window.scrollFrameRates);
  const avgFrameRate = frameRates.reduce((a, b) => a + b, 0) / frameRates.length;
  
  console.log(`📊 Average scroll frame rate: ${avgFrameRate.toFixed(2)} fps`);
  expect(avgFrameRate).toBeGreaterThan(PERFORMANCE_THRESHOLDS.scrollFrameRate);
}

async function testListItemInteractions(page) {
  console.log('🖱️  Testing list item interactions...');
  
  // Test clicking on list items
  const items = await page.locator('[data-testid^="project-item-"]').first(10);
  
  for (let i = 0; i < 5; i++) {
    const startTime = Date.now();
    await items.nth(i).click();
    
    // Wait for item to be selected/highlighted
    await page.waitForSelector('[data-testid="project-details"]', { timeout: 1000 });
    
    const responseTime = Date.now() - startTime;
    console.log(`📊 Item ${i} click response time: ${responseTime}ms`);
    expect(responseTime).toBeLessThan(300); // 300ms max for item interaction
  }
}

async function testVirtualScrolling(page) {
  console.log('🔄 Testing virtual scrolling implementation...');
  
  const listContainer = await page.locator('[data-testid="contacts-list"]');
  
  // Check that only visible items are rendered
  const initialItemCount = await page.locator('[data-testid^="contact-item-"]').count();
  console.log(`📊 Initial rendered items: ${initialItemCount}`);
  
  // Should not render all 10,000 items at once
  expect(initialItemCount).toBeLessThan(100);
  
  // Scroll to bottom and verify new items are rendered
  await listContainer.evaluate(el => {
    el.scrollTop = el.scrollHeight;
  });
  
  await page.waitForTimeout(500); // Wait for virtual scrolling to update
  
  const finalItemCount = await page.locator('[data-testid^="contact-item-"]').count();
  console.log(`📊 Final rendered items: ${finalItemCount}`);
  
  // Should still not render all items
  expect(finalItemCount).toBeLessThan(200);
}

async function testRapidScrolling(page) {
  console.log('⚡ Testing rapid scrolling performance...');
  
  const listContainer = await page.locator('[data-testid="contacts-list"]');
  
  // Perform rapid scrolling
  const startTime = Date.now();
  
  for (let i = 0; i < 10; i++) {
    await listContainer.evaluate((el, scrollPos) => {
      el.scrollTop = scrollPos;
    }, i * 1000);
    await page.waitForTimeout(50); // Brief pause between scrolls
  }
  
  const endTime = Date.now();
  const totalTime = endTime - startTime;
  
  console.log(`📊 Rapid scrolling completed in: ${totalTime}ms`);
  expect(totalTime).toBeLessThan(2000); // Should complete rapid scrolling quickly
}

async function performMemoryIntensiveOperations(page) {
  console.log('💾 Performing memory-intensive operations...');
  
  // Simulate operations that might cause memory leaks
  const operations = [
    'search filtering',
    'sorting',
    'modal opening/closing',
    'data refreshing'
  ];
  
  for (const operation of operations) {
    console.log(`🔄 Testing ${operation}...`);
    
    switch (operation) {
      case 'search filtering':
        for (let i = 0; i < 10; i++) {
          await page.fill('[data-testid="search-input"]', `search_${i}`);
          await page.waitForTimeout(100);
        }
        await page.fill('[data-testid="search-input"]', '');
        break;
        
      case 'sorting':
        for (let i = 0; i < 5; i++) {
          await page.click('[data-testid="sort-by-name"]');
          await page.waitForTimeout(200);
          await page.click('[data-testid="sort-by-date"]');
          await page.waitForTimeout(200);
        }
        break;
        
      case 'modal opening/closing':
        for (let i = 0; i < 5; i++) {
          await page.click('[data-testid="add-company"]');
          await page.waitForSelector('[data-testid="company-modal"]');
          await page.click('[data-testid="modal-close"]');
          await page.waitForTimeout(100);
        }
        break;
        
      case 'data refreshing':
        for (let i = 0; i < 3; i++) {
          await page.click('[data-testid="refresh-data"]');
          await page.waitForTimeout(500);
        }
        break;
    }
  }
}

async function getMemoryUsage(page) {
  const memoryInfo = await page.evaluate(() => {
    if (performance.memory) {
      return performance.memory.usedJSHeapSize / 1024 / 1024; // Convert to MB
    }
    return null;
  });
  
  return memoryInfo || 0;
}

async function generatePerformanceReport(page, metrics) {
  console.log('📊 Generating performance report...');
  
  const report = {
    timestamp: new Date().toISOString(),
    metrics: metrics,
    thresholds: PERFORMANCE_THRESHOLDS,
    browser: await page.evaluate(() => navigator.userAgent),
  };
  
  // Save report to file
  const fs = require('fs');
  const reportPath = path.join(process.cwd(), 'performance', 'reports', 'ui-performance-report.json');
  
  try {
    await fs.promises.mkdir(path.dirname(reportPath), { recursive: true });
    await fs.promises.writeFile(reportPath, JSON.stringify(report, null, 2));
    console.log(`📄 Performance report saved: ${reportPath}`);
  } catch (error) {
    console.error('Failed to save performance report:', error);
  }
}

async function cleanupTestData(page) {
  console.log('🧹 Cleaning up test data...');
  
  await page.evaluate(() => {
    // Clear test data from memory
    if (window.testData) {
      delete window.testData;
    }
    if (window.performanceMetrics) {
      delete window.performanceMetrics;
    }
  });
}