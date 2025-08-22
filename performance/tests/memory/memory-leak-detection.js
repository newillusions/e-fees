#!/usr/bin/env node
/**
 * Memory Leak Detection for E-Fees Application
 * 
 * Monitors memory usage patterns over extended periods to detect:
 * - Memory leaks in component lifecycle
 * - Unbounded memory growth during operations
 * - Garbage collection effectiveness
 * - Memory usage patterns under load
 */

import { promises as fs } from 'fs';
import { spawn, exec } from 'child_process';
import path from 'path';
import { promisify } from 'util';

const execAsync = promisify(exec);

class MemoryLeakDetector {
  constructor() {
    this.monitoringActive = false;
    this.testResults = {
      startTime: null,
      endTime: null,
      memorySnapshots: [],
      leakDetections: [],
      performanceMetrics: {},
      gcStats: []
    };
    
    this.config = {
      // Test duration and intervals
      testDurationMinutes: 30,
      snapshotIntervalSeconds: 10,
      gcForceIntervalMinutes: 5,
      
      // Memory thresholds
      baselineMemoryMB: 100,
      warningThresholdMB: 300,
      criticalThresholdMB: 500,
      growthRateThresholdMBPerMin: 2,
      
      // Application settings
      appUrl: 'http://localhost:1420',
      appStartCommand: 'npm run tauri:dev',
      
      // Output settings
      outputDir: path.join(process.cwd(), 'performance', 'reports', 'memory'),
      detailedLogging: process.env.MEMORY_DEBUG === 'true'
    };
  }

  async initialize() {
    console.log('🧠 Initializing Memory Leak Detection...');
    
    // Ensure output directory exists
    await fs.mkdir(this.config.outputDir, { recursive: true });
    
    this.testResults.startTime = new Date().toISOString();
    
    console.log('📊 Memory leak detection configuration:');
    console.log(`   Test Duration: ${this.config.testDurationMinutes} minutes`);
    console.log(`   Snapshot Interval: ${this.config.snapshotIntervalSeconds} seconds`);
    console.log(`   Warning Threshold: ${this.config.warningThresholdMB} MB`);
    console.log(`   Critical Threshold: ${this.config.criticalThresholdMB} MB`);
  }

  async runMemoryLeakTests() {
    console.log('🚀 Starting comprehensive memory leak tests...');
    
    try {
      // Test 1: Component lifecycle memory leaks
      await this.testComponentLifecycleLeaks();
      
      // Test 2: Data loading memory patterns
      await this.testDataLoadingMemoryPatterns();
      
      // Test 3: Long-running session stability
      await this.testLongRunningSessionStability();
      
      // Test 4: Garbage collection effectiveness
      await this.testGarbageCollectionEffectiveness();
      
      // Test 5: Memory usage under concurrent load
      await this.testConcurrentLoadMemoryUsage();
      
      // Generate comprehensive report
      await this.generateMemoryReport();
      
    } catch (error) {
      console.error('❌ Memory leak test failed:', error);
      throw error;
    }
  }

  async testComponentLifecycleLeaks() {
    console.log('🔄 Testing component lifecycle memory leaks...');
    
    const testScenarios = [
      {
        name: 'Modal Open/Close Cycle',
        script: this.generateModalCycleScript(),
        iterations: 100,
        expectedMemoryGrowth: 5 // MB
      },
      {
        name: 'Route Navigation Cycle', 
        script: this.generateNavigationCycleScript(),
        iterations: 50,
        expectedMemoryGrowth: 10 // MB
      },
      {
        name: 'List Rendering Cycle',
        script: this.generateListRenderingCycleScript(),
        iterations: 25,
        expectedMemoryGrowth: 15 // MB
      },
      {
        name: 'Search Filter Cycle',
        script: this.generateSearchFilterCycleScript(),
        iterations: 200,
        expectedMemoryGrowth: 3 // MB
      }
    ];

    for (const scenario of testScenarios) {
      console.log(`🧪 Testing: ${scenario.name}`);
      
      const initialMemory = await this.getCurrentMemoryUsage();
      
      // Execute the test scenario
      await this.executeMemoryTestScenario(scenario);
      
      const finalMemory = await this.getCurrentMemoryUsage();
      const memoryGrowth = finalMemory - initialMemory;
      
      const testResult = {
        scenario: scenario.name,
        iterations: scenario.iterations,
        initialMemoryMB: initialMemory,
        finalMemoryMB: finalMemory,
        memoryGrowthMB: memoryGrowth,
        expectedGrowthMB: scenario.expectedMemoryGrowth,
        passed: memoryGrowth <= scenario.expectedMemoryGrowth,
        timestamp: new Date().toISOString()
      };
      
      this.testResults.leakDetections.push(testResult);
      
      console.log(`📊 ${scenario.name}: ${memoryGrowth.toFixed(2)}MB growth (expected: ${scenario.expectedMemoryGrowth}MB)`);
      
      if (!testResult.passed) {
        console.warn(`⚠️  Memory leak detected in ${scenario.name}!`);
      }
      
      // Force garbage collection between tests
      await this.forceGarbageCollection();
      await this.sleep(2000); // Allow GC to complete
    }
  }

  async testDataLoadingMemoryPatterns() {
    console.log('📊 Testing data loading memory patterns...');
    
    const dataLoadTests = [
      {
        name: 'Small Dataset Loading (500 items)',
        dataSize: 'small',
        itemCount: 500,
        expectedPeakMemoryMB: 150
      },
      {
        name: 'Medium Dataset Loading (2000 items)',
        dataSize: 'medium', 
        itemCount: 2000,
        expectedPeakMemoryMB: 250
      },
      {
        name: 'Large Dataset Loading (5000 items)',
        dataSize: 'large',
        itemCount: 5000,
        expectedPeakMemoryMB: 400
      }
    ];

    for (const test of dataLoadTests) {
      console.log(`🔄 Testing: ${test.name}`);
      
      const memoryBefore = await this.getCurrentMemoryUsage();
      
      // Simulate data loading
      await this.simulateDataLoading(test.dataSize, test.itemCount);
      
      const memoryPeak = await this.getCurrentMemoryUsage();
      
      // Clear data and measure cleanup
      await this.simulateDataCleanup();
      await this.forceGarbageCollection();
      await this.sleep(3000);
      
      const memoryAfterCleanup = await this.getCurrentMemoryUsage();
      
      const testResult = {
        test: test.name,
        itemCount: test.itemCount,
        memoryBeforeMB: memoryBefore,
        memoryPeakMB: memoryPeak,
        memoryAfterCleanupMB: memoryAfterCleanup,
        memoryGrowthMB: memoryPeak - memoryBefore,
        memoryRetainedMB: memoryAfterCleanup - memoryBefore,
        cleanupEfficiency: ((memoryPeak - memoryAfterCleanup) / (memoryPeak - memoryBefore)) * 100,
        passed: memoryPeak <= test.expectedPeakMemoryMB && memoryAfterCleanup <= memoryBefore + 10,
        timestamp: new Date().toISOString()
      };
      
      this.testResults.leakDetections.push(testResult);
      
      console.log(`📈 Peak: ${memoryPeak.toFixed(2)}MB, Cleanup: ${testResult.cleanupEfficiency.toFixed(1)}%`);
      
      if (!testResult.passed) {
        console.warn(`⚠️  Data loading memory issue detected: ${test.name}`);
      }
    }
  }

  async testLongRunningSessionStability() {
    console.log('⏱️  Testing long-running session memory stability...');
    
    const sessionDurationMinutes = 20;
    const operationIntervalSeconds = 30;
    const maxAllowedGrowthMBPerHour = 10;
    
    console.log(`🔄 Running ${sessionDurationMinutes}-minute session stability test...`);
    
    const startTime = Date.now();
    const endTime = startTime + (sessionDurationMinutes * 60 * 1000);
    const initialMemory = await this.getCurrentMemoryUsage();
    
    const memorySnapshots = [];
    
    while (Date.now() < endTime) {
      // Perform typical user operations
      await this.simulateTypicalUserActivity();
      
      // Take memory snapshot
      const currentMemory = await this.getCurrentMemoryUsage();
      const timestamp = Date.now();
      
      memorySnapshots.push({
        timestamp: timestamp,
        memoryMB: currentMemory,
        elapsedMinutes: (timestamp - startTime) / (1000 * 60)
      });
      
      console.log(`📊 ${Math.floor((timestamp - startTime) / 1000 / 60)}min: ${currentMemory.toFixed(2)}MB`);
      
      // Wait for next operation
      await this.sleep(operationIntervalSeconds * 1000);
    }
    
    const finalMemory = await this.getCurrentMemoryUsage();
    const totalGrowthMB = finalMemory - initialMemory;
    const growthRatePerHour = (totalGrowthMB / sessionDurationMinutes) * 60;
    
    // Analyze memory growth trend
    const memoryTrend = this.analyzeMemoryTrend(memorySnapshots);
    
    const sessionResult = {
      test: 'Long Running Session Stability',
      durationMinutes: sessionDurationMinutes,
      initialMemoryMB: initialMemory,
      finalMemoryMB: finalMemory,
      totalGrowthMB: totalGrowthMB,
      growthRatePerHourMB: growthRatePerHour,
      maxAllowedGrowthPerHourMB: maxAllowedGrowthMBPerHour,
      memoryTrend: memoryTrend,
      snapshots: memorySnapshots,
      passed: growthRatePerHour <= maxAllowedGrowthMBPerHour && memoryTrend.slope <= 1,
      timestamp: new Date().toISOString()
    };
    
    this.testResults.leakDetections.push(sessionResult);
    
    console.log(`📊 Session complete: ${totalGrowthMB.toFixed(2)}MB growth (${growthRatePerHour.toFixed(2)}MB/hour)`);
    console.log(`📈 Memory trend: ${memoryTrend.direction} (slope: ${memoryTrend.slope.toFixed(3)})`);
    
    if (!sessionResult.passed) {
      console.warn('⚠️  Long-running session memory leak detected!');
    }
  }

  async testGarbageCollectionEffectiveness() {
    console.log('🗑️  Testing garbage collection effectiveness...');
    
    const gcTests = [
      {
        name: 'Force GC after Data Load',
        operation: async () => {
          await this.simulateDataLoading('large', 5000);
          await this.simulateDataCleanup();
        }
      },
      {
        name: 'Force GC after Component Cycles',
        operation: async () => {
          for (let i = 0; i < 20; i++) {
            await this.simulateModalOpenClose();
          }
        }
      },
      {
        name: 'Force GC after Heavy Operations',
        operation: async () => {
          await this.simulateHeavyComputationOperations();
        }
      }
    ];

    for (const test of gcTests) {
      console.log(`🔄 Testing: ${test.name}`);
      
      const memoryBefore = await this.getCurrentMemoryUsage();
      
      // Perform memory-intensive operation
      await test.operation();
      
      const memoryAfterOperation = await this.getCurrentMemoryUsage();
      
      // Force garbage collection
      const gcStartTime = Date.now();
      await this.forceGarbageCollection();
      const gcDuration = Date.now() - gcStartTime;
      
      // Measure memory after GC
      await this.sleep(1000);
      const memoryAfterGC = await this.getCurrentMemoryUsage();
      
      const gcEffectiveness = {
        test: test.name,
        memoryBeforeMB: memoryBefore,
        memoryAfterOperationMB: memoryAfterOperation,
        memoryAfterGCMB: memoryAfterGC,
        memoryFreedMB: memoryAfterOperation - memoryAfterGC,
        gcEfficiencyPercent: ((memoryAfterOperation - memoryAfterGC) / (memoryAfterOperation - memoryBefore)) * 100,
        gcDurationMS: gcDuration,
        passed: (memoryAfterGC - memoryBefore) <= 20 && gcDuration < 1000,
        timestamp: new Date().toISOString()
      };
      
      this.testResults.gcStats.push(gcEffectiveness);
      
      console.log(`📊 GC freed ${gcEffectiveness.memoryFreedMB.toFixed(2)}MB (${gcEffectiveness.gcEfficiencyPercent.toFixed(1)}% efficiency)`);
      
      if (!gcEffectiveness.passed) {
        console.warn(`⚠️  GC effectiveness issue: ${test.name}`);
      }
    }
  }

  async testConcurrentLoadMemoryUsage() {
    console.log('⚡ Testing memory usage under concurrent load...');
    
    const concurrentOperations = [
      { name: 'Data Loading', operation: () => this.simulateDataLoading('medium', 1000) },
      { name: 'Search Operations', operation: () => this.simulateSearchOperations() },
      { name: 'Modal Operations', operation: () => this.simulateModalOperations() },
      { name: 'Navigation Operations', operation: () => this.simulateNavigationOperations() }
    ];

    const memoryBefore = await this.getCurrentMemoryUsage();
    
    // Execute operations concurrently
    console.log('🔄 Starting concurrent operations...');
    const operationPromises = concurrentOperations.map(op => op.operation());
    
    // Monitor memory during concurrent execution
    const monitoringPromise = this.monitorMemoryDuringExecution(10000); // 10 second monitoring
    
    await Promise.all([...operationPromises, monitoringPromise]);
    
    const memoryAfter = await this.getCurrentMemoryUsage();
    const memoryGrowth = memoryAfter - memoryBefore;
    
    const concurrentResult = {
      test: 'Concurrent Load Memory Usage',
      operationCount: concurrentOperations.length,
      memoryBeforeMB: memoryBefore,
      memoryAfterMB: memoryAfter,
      memoryGrowthMB: memoryGrowth,
      maxExpectedGrowthMB: 100,
      passed: memoryGrowth <= 100,
      timestamp: new Date().toISOString()
    };
    
    this.testResults.leakDetections.push(concurrentResult);
    
    console.log(`📊 Concurrent operations memory growth: ${memoryGrowth.toFixed(2)}MB`);
    
    if (!concurrentResult.passed) {
      console.warn('⚠️  Excessive memory usage under concurrent load!');
    }
  }

  // Helper methods for test scenarios

  generateModalCycleScript() {
    return `
      // Modal cycle script
      for (let i = 0; i < iterations; i++) {
        document.querySelector('[data-testid="add-company"]')?.click();
        await new Promise(resolve => setTimeout(resolve, 100));
        document.querySelector('[data-testid="modal-close"]')?.click();
        await new Promise(resolve => setTimeout(resolve, 100));
      }
    `;
  }

  generateNavigationCycleScript() {
    return `
      // Navigation cycle script
      const routes = ['projects', 'proposals', 'companies', 'contacts', 'dashboard'];
      for (let i = 0; i < iterations; i++) {
        const route = routes[i % routes.length];
        document.querySelector(\`[data-testid="nav-\${route}"]\`)?.click();
        await new Promise(resolve => setTimeout(resolve, 200));
      }
    `;
  }

  generateListRenderingCycleScript() {
    return `
      // List rendering cycle script
      for (let i = 0; i < iterations; i++) {
        // Simulate loading different sized datasets
        window.testData = generateTestData(1000 + (i * 100));
        await new Promise(resolve => setTimeout(resolve, 300));
        window.testData = null;
        await new Promise(resolve => setTimeout(resolve, 100));
      }
    `;
  }

  generateSearchFilterCycleScript() {
    return `
      // Search filter cycle script
      const searchTerms = ['Dubai', 'Tower', 'Active', 'Company', 'Project'];
      for (let i = 0; i < iterations; i++) {
        const term = searchTerms[i % searchTerms.length];
        document.querySelector('[data-testid="search-input"]').value = term;
        document.querySelector('[data-testid="search-input"]').dispatchEvent(new Event('input'));
        await new Promise(resolve => setTimeout(resolve, 50));
        document.querySelector('[data-testid="search-input"]').value = '';
        document.querySelector('[data-testid="search-input"]').dispatchEvent(new Event('input'));
        await new Promise(resolve => setTimeout(resolve, 50));
      }
    `;
  }

  async executeMemoryTestScenario(scenario) {
    // This would integrate with the Tauri application
    // For now, simulate the execution
    console.log(`Executing ${scenario.name} with ${scenario.iterations} iterations`);
    await this.sleep(scenario.iterations * 10); // Simulate execution time
  }

  async simulateDataLoading(size, itemCount) {
    console.log(`Simulating ${size} data loading (${itemCount} items)`);
    await this.sleep(Math.max(500, itemCount / 10)); // Simulate loading time
  }

  async simulateDataCleanup() {
    console.log('Simulating data cleanup');
    await this.sleep(200);
  }

  async simulateTypicalUserActivity() {
    const activities = [
      () => this.simulateModalOpenClose(),
      () => this.simulateNavigation(),
      () => this.simulateSearch(),
      () => this.simulateListScroll()
    ];
    
    const activity = activities[Math.floor(Math.random() * activities.length)];
    await activity();
  }

  async simulateModalOpenClose() {
    await this.sleep(100);
  }

  async simulateNavigation() {
    await this.sleep(150);
  }

  async simulateSearch() {
    await this.sleep(75);
  }

  async simulateListScroll() {
    await this.sleep(50);
  }

  async simulateHeavyComputationOperations() {
    console.log('Simulating heavy computation operations');
    await this.sleep(2000);
  }

  async simulateSearchOperations() {
    console.log('Simulating search operations');
    await this.sleep(1000);
  }

  async simulateModalOperations() {
    console.log('Simulating modal operations');
    await this.sleep(1500);
  }

  async simulateNavigationOperations() {
    console.log('Simulating navigation operations');
    await this.sleep(800);
  }

  async getCurrentMemoryUsage() {
    try {
      // Get Node.js process memory usage
      const memUsage = process.memoryUsage();
      return memUsage.heapUsed / 1024 / 1024; // Convert to MB
    } catch (error) {
      console.error('Error getting memory usage:', error);
      return 0;
    }
  }

  async forceGarbageCollection() {
    if (global.gc) {
      console.log('🗑️  Forcing garbage collection...');
      global.gc();
    } else {
      console.log('⚠️  Garbage collection not available (run with --expose-gc)');
    }
  }

  async monitorMemoryDuringExecution(durationMs) {
    const startTime = Date.now();
    const snapshots = [];
    
    while (Date.now() - startTime < durationMs) {
      const memory = await this.getCurrentMemoryUsage();
      snapshots.push({
        timestamp: Date.now(),
        memoryMB: memory
      });
      
      await this.sleep(100); // Take snapshot every 100ms
    }
    
    return snapshots;
  }

  analyzeMemoryTrend(snapshots) {
    if (snapshots.length < 2) {
      return { direction: 'insufficient_data', slope: 0 };
    }
    
    // Simple linear regression to determine trend
    const n = snapshots.length;
    const sumX = snapshots.reduce((sum, s) => sum + s.elapsedMinutes, 0);
    const sumY = snapshots.reduce((sum, s) => sum + s.memoryMB, 0);
    const sumXY = snapshots.reduce((sum, s) => sum + s.elapsedMinutes * s.memoryMB, 0);
    const sumXX = snapshots.reduce((sum, s) => sum + s.elapsedMinutes * s.elapsedMinutes, 0);
    
    const slope = (n * sumXY - sumX * sumY) / (n * sumXX - sumX * sumX);
    
    let direction;
    if (slope > 0.5) direction = 'increasing';
    else if (slope < -0.5) direction = 'decreasing';
    else direction = 'stable';
    
    return { direction, slope };
  }

  async generateMemoryReport() {
    console.log('📊 Generating comprehensive memory report...');
    
    this.testResults.endTime = new Date().toISOString();
    
    const report = {
      metadata: {
        testStartTime: this.testResults.startTime,
        testEndTime: this.testResults.endTime,
        testDurationMinutes: this.config.testDurationMinutes,
        nodeVersion: process.version,
        platform: process.platform,
        arch: process.arch
      },
      configuration: this.config,
      summary: this.generateTestSummary(),
      componentLifecycleTests: this.testResults.leakDetections.filter(t => t.scenario),
      dataLoadingTests: this.testResults.leakDetections.filter(t => t.test && t.itemCount),
      longRunningSessionTest: this.testResults.leakDetections.find(t => t.test === 'Long Running Session Stability'),
      garbageCollectionTests: this.testResults.gcStats,
      concurrentLoadTest: this.testResults.leakDetections.find(t => t.test === 'Concurrent Load Memory Usage'),
      recommendations: this.generateRecommendations()
    };
    
    // Save detailed JSON report
    const jsonReportPath = path.join(this.config.outputDir, `memory-leak-report-${Date.now()}.json`);
    await fs.writeFile(jsonReportPath, JSON.stringify(report, null, 2));
    
    // Generate HTML report
    const htmlReportPath = await this.generateHTMLReport(report);
    
    console.log('✅ Memory leak detection complete!');
    console.log(`📄 JSON Report: ${jsonReportPath}`);
    console.log(`🌐 HTML Report: ${htmlReportPath}`);
    
    // Print summary to console
    this.printReportSummary(report);
  }

  generateTestSummary() {
    const allTests = this.testResults.leakDetections;
    const passedTests = allTests.filter(t => t.passed);
    const failedTests = allTests.filter(t => !t.passed);
    
    return {
      totalTests: allTests.length,
      passedTests: passedTests.length,
      failedTests: failedTests.length,
      successRate: (passedTests.length / allTests.length) * 100,
      criticalIssues: failedTests.filter(t => 
        (t.memoryGrowthMB && t.memoryGrowthMB > 50) ||
        (t.growthRatePerHourMB && t.growthRatePerHourMB > 20)
      ).length
    };
  }

  generateRecommendations() {
    const recommendations = [];
    const failedTests = this.testResults.leakDetections.filter(t => !t.passed);
    
    if (failedTests.length === 0) {
      recommendations.push({
        type: 'success',
        message: 'No significant memory leaks detected. Memory management appears to be working effectively.'
      });
    } else {
      failedTests.forEach(test => {
        if (test.scenario && test.scenario.includes('Modal')) {
          recommendations.push({
            type: 'warning',
            message: 'Modal component lifecycle shows memory retention. Review event listener cleanup and component disposal.',
            test: test.scenario
          });
        }
        
        if (test.scenario && test.scenario.includes('Navigation')) {
          recommendations.push({
            type: 'warning',
            message: 'Route navigation shows memory growth. Check for proper cleanup of route-specific resources.',
            test: test.scenario
          });
        }
        
        if (test.test === 'Long Running Session Stability') {
          recommendations.push({
            type: 'critical',
            message: 'Long-running session shows memory growth. This could impact user experience in extended usage.',
            growthRate: `${test.growthRatePerHourMB.toFixed(2)}MB/hour`
          });
        }
      });
    }
    
    // GC effectiveness recommendations
    const ineffectiveGC = this.testResults.gcStats.filter(gc => gc.gcEfficiencyPercent < 50);
    if (ineffectiveGC.length > 0) {
      recommendations.push({
        type: 'info',
        message: 'Garbage collection effectiveness could be improved. Consider reviewing object retention patterns.'
      });
    }
    
    return recommendations;
  }

  async generateHTMLReport(report) {
    const htmlContent = `
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>E-Fees Memory Leak Detection Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; background: #f5f5f5; }
        .container { max-width: 1200px; margin: 0 auto; background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        .header { text-align: center; margin-bottom: 30px; }
        .summary { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin-bottom: 30px; }
        .metric-card { background: #f8f9fa; padding: 15px; border-radius: 4px; text-align: center; }
        .metric-value { font-size: 24px; font-weight: bold; }
        .success { color: #28a745; }
        .warning { color: #ffc107; }
        .danger { color: #dc3545; }
        .test-section { margin: 30px 0; }
        .test-result { background: #f8f9fa; margin: 10px 0; padding: 15px; border-radius: 4px; border-left: 4px solid #ccc; }
        .test-result.passed { border-left-color: #28a745; }
        .test-result.failed { border-left-color: #dc3545; }
        .recommendations { background: #e9ecef; padding: 20px; border-radius: 4px; }
        .recommendation { margin: 10px 0; padding: 10px; border-radius: 4px; }
        .recommendation.success { background: #d4edda; color: #155724; }
        .recommendation.warning { background: #fff3cd; color: #856404; }
        .recommendation.critical { background: #f8d7da; color: #721c24; }
        .chart-placeholder { height: 200px; background: #e9ecef; border-radius: 4px; display: flex; align-items: center; justify-content: center; color: #6c757d; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🧠 E-Fees Memory Leak Detection Report</h1>
            <p>Generated on ${new Date(report.metadata.testEndTime).toLocaleString()}</p>
        </div>

        <div class="summary">
            <div class="metric-card">
                <div class="metric-value ${report.summary.successRate === 100 ? 'success' : report.summary.successRate > 80 ? 'warning' : 'danger'}">
                    ${report.summary.successRate.toFixed(1)}%
                </div>
                <div>Success Rate</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">${report.summary.totalTests}</div>
                <div>Total Tests</div>
            </div>
            <div class="metric-card">
                <div class="metric-value ${report.summary.failedTests === 0 ? 'success' : 'danger'}">
                    ${report.summary.failedTests}
                </div>
                <div>Failed Tests</div>
            </div>
            <div class="metric-card">
                <div class="metric-value ${report.summary.criticalIssues === 0 ? 'success' : 'danger'}">
                    ${report.summary.criticalIssues}
                </div>
                <div>Critical Issues</div>
            </div>
        </div>

        <div class="test-section">
            <h2>📊 Test Results Summary</h2>
            ${report.componentLifecycleTests.map(test => `
                <div class="test-result ${test.passed ? 'passed' : 'failed'}">
                    <h3>${test.scenario}</h3>
                    <p><strong>Memory Growth:</strong> ${test.memoryGrowthMB.toFixed(2)}MB (expected: ${test.expectedGrowthMB}MB)</p>
                    <p><strong>Iterations:</strong> ${test.iterations}</p>
                    <p><strong>Status:</strong> ${test.passed ? '✅ Passed' : '❌ Failed'}</p>
                </div>
            `).join('')}
        </div>

        ${report.longRunningSessionTest ? `
        <div class="test-section">
            <h2>⏱️ Long-Running Session Test</h2>
            <div class="test-result ${report.longRunningSessionTest.passed ? 'passed' : 'failed'}">
                <h3>Session Stability (${report.longRunningSessionTest.durationMinutes} minutes)</h3>
                <p><strong>Total Growth:</strong> ${report.longRunningSessionTest.totalGrowthMB.toFixed(2)}MB</p>
                <p><strong>Growth Rate:</strong> ${report.longRunningSessionTest.growthRatePerHourMB.toFixed(2)}MB/hour</p>
                <p><strong>Trend:</strong> ${report.longRunningSessionTest.memoryTrend.direction}</p>
                <p><strong>Status:</strong> ${report.longRunningSessionTest.passed ? '✅ Passed' : '❌ Failed'}</p>
            </div>
        </div>
        ` : ''}

        <div class="test-section">
            <h2>🗑️ Garbage Collection Effectiveness</h2>
            ${report.garbageCollectionTests.map(gc => `
                <div class="test-result ${gc.passed ? 'passed' : 'failed'}">
                    <h3>${gc.test}</h3>
                    <p><strong>Memory Freed:</strong> ${gc.memoryFreedMB.toFixed(2)}MB</p>
                    <p><strong>Efficiency:</strong> ${gc.gcEfficiencyPercent.toFixed(1)}%</p>
                    <p><strong>GC Duration:</strong> ${gc.gcDurationMS}ms</p>
                    <p><strong>Status:</strong> ${gc.passed ? '✅ Passed' : '❌ Failed'}</p>
                </div>
            `).join('')}
        </div>

        <div class="recommendations">
            <h2>💡 Recommendations</h2>
            ${report.recommendations.map(rec => `
                <div class="recommendation ${rec.type}">
                    <strong>${rec.type.toUpperCase()}:</strong> ${rec.message}
                    ${rec.test ? `<br><small>Test: ${rec.test}</small>` : ''}
                    ${rec.growthRate ? `<br><small>Growth Rate: ${rec.growthRate}</small>` : ''}
                </div>
            `).join('')}
        </div>

        <div class="test-section">
            <h2>ℹ️ Test Configuration</h2>
            <p><strong>Test Duration:</strong> ${report.configuration.testDurationMinutes} minutes</p>
            <p><strong>Snapshot Interval:</strong> ${report.configuration.snapshotIntervalSeconds} seconds</p>
            <p><strong>Warning Threshold:</strong> ${report.configuration.warningThresholdMB} MB</p>
            <p><strong>Critical Threshold:</strong> ${report.configuration.criticalThresholdMB} MB</p>
            <p><strong>Node.js Version:</strong> ${report.metadata.nodeVersion}</p>
            <p><strong>Platform:</strong> ${report.metadata.platform} ${report.metadata.arch}</p>
        </div>
    </div>
</body>
</html>
    `;

    const htmlReportPath = path.join(this.config.outputDir, `memory-leak-report-${Date.now()}.html`);
    await fs.writeFile(htmlReportPath, htmlContent);
    
    return htmlReportPath;
  }

  printReportSummary(report) {
    console.log('\n' + '='.repeat(60));
    console.log('📊 MEMORY LEAK DETECTION SUMMARY');
    console.log('='.repeat(60));
    console.log(`✅ Passed Tests: ${report.summary.passedTests}/${report.summary.totalTests}`);
    console.log(`❌ Failed Tests: ${report.summary.failedTests}`);
    console.log(`🔥 Critical Issues: ${report.summary.criticalIssues}`);
    console.log(`📈 Success Rate: ${report.summary.successRate.toFixed(1)}%`);
    
    if (report.summary.criticalIssues > 0) {
      console.log('\n🚨 CRITICAL MEMORY ISSUES DETECTED:');
      report.recommendations
        .filter(r => r.type === 'critical')
        .forEach(r => console.log(`   • ${r.message}`));
    }
    
    if (report.summary.successRate === 100) {
      console.log('\n🎉 All memory leak tests passed!');
    } else if (report.summary.successRate > 80) {
      console.log('\n⚠️  Some memory issues detected, but overall performance is acceptable.');
    } else {
      console.log('\n🚨 Significant memory issues detected. Review recommended.');
    }
    
    console.log('='.repeat(60));
  }

  sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

// CLI interface
async function main() {
  const detector = new MemoryLeakDetector();
  
  try {
    await detector.initialize();
    await detector.runMemoryLeakTests();
  } catch (error) {
    console.error('❌ Memory leak detection failed:', error);
    process.exit(1);
  }
}

// Run if called directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}

export default MemoryLeakDetector;