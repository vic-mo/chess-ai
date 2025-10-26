# M8: WASM Integration - Cross-Browser Testing Guide

## Overview

This guide covers manual cross-browser testing procedures for the WASM chess engine integration. Use this guide to verify functionality across different browsers, platforms, and devices.

## Pre-Testing Setup

### 1. Build the Project

```bash
# Build WASM module
pnpm build:wasm

# Start dev server
pnpm dev
```

The app will be available at `http://localhost:5173`

### 2. Access Browser Compatibility Info

The app automatically detects your browser and shows compatibility warnings. Click "Show Details" on the compatibility banner to see:

- Browser name and version
- Operating system
- Feature support (WASM, Workers, SharedArrayBuffer)
- Any warnings or errors

## Desktop Browser Testing

### Chrome (Latest)

**Platforms:** Windows, macOS, Linux

**Test Steps:**

1. Open `http://localhost:5173`
2. Verify no compatibility warnings appear
3. Select "WASM (Local)" mode
4. Verify status shows "ready"
5. Enter FEN: `rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1`
6. Set depth: `8`
7. Click "Analyze"
8. Verify:
   - Search info updates appear (depth 1, 2, 3...)
   - Each update shows nodes, NPS, score, PV
   - Final bestmove appears
   - Status returns to "ready"
9. Click "Show Performance"
10. Verify metrics are tracked:
    - WASM load time
    - Worker create time
    - Search count
    - Average search time

**Expected Performance:**

- WASM load: <100ms (first load), <10ms (cached)
- Worker create: <10ms
- Search (depth 8): <100ms

**Known Issues:** None

---

### Firefox (Latest)

**Platforms:** Windows, macOS, Linux

**Test Steps:**

1. Same as Chrome test steps
2. Additionally check:
   - Console for any warnings
   - WASM compilation time (may be slower than Chrome)

**Expected Performance:**

- WASM load: <150ms (first load)
- Worker create: <10ms
- Search (depth 8): <150ms (slightly slower than Chrome)

**Known Issues:**

- Firefox <79 may have slower WASM performance
- Compatibility warning will appear for older versions

---

### Safari (Latest)

**Platforms:** macOS

**Test Steps:**

1. Same as Chrome test steps
2. Additionally check:
   - Performance Memory API warning (expected)
   - WASM load time (may be slower)

**Expected Performance:**

- WASM load: <150ms (first load)
- Worker create: <15ms
- Search (depth 8): <150ms

**Known Issues:**

- Performance Memory API not available (warning expected)
- Memory tracking will not work

---

### Edge (Latest)

**Platforms:** Windows, macOS

**Test Steps:**

1. Same as Chrome test steps (Edge uses Chromium)

**Expected Performance:**

- Same as Chrome

**Known Issues:** None

---

## Mobile Browser Testing

### iOS Safari

**Devices:** iPhone, iPad (iOS 14+)

**Test Steps:**

1. Open `http://[YOUR_LOCAL_IP]:5173` on device
   - Find your local IP: `ipconfig` (Windows) or `ifconfig` (Mac/Linux)
   - Ensure device is on same network
2. Verify compatibility warning appears:
   - "iOS Safari may have reduced WASM performance due to JIT limitations"
3. Select "WASM (Local)" mode
4. Verify status shows "ready"
5. Test analysis with simple position
6. Verify search completes (may be slower)

**Expected Performance:**

- WASM load: <200ms
- Worker create: <20ms
- Search (depth 8): <300ms (significantly slower due to JIT limits)

**Known Issues:**

- Reduced WASM performance (warning expected)
- Performance Memory API not available
- May throttle in background tabs

**Alternative Test:** Use fake mode to verify UI works correctly

---

### Chrome Android

**Devices:** Android 8+

**Test Steps:**

1. Open `http://[YOUR_LOCAL_IP]:5173` on device
2. Verify no compatibility warnings (or only SharedArrayBuffer warning)
3. Select "WASM (Local)" mode
4. Test analysis with simple position

**Expected Performance:**

- WASM load: <150ms
- Worker create: <15ms
- Search (depth 8): <200ms

**Known Issues:**

- Performance varies widely by device
- May throttle in background tabs

---

## Feature-Specific Testing

### Test 1: Mode Switching

**Objective:** Verify clean transitions between engine modes

**Steps:**

1. Start in "Fake (Demo)" mode
2. Run analysis
3. Switch to "WASM (Local)" mode
4. Wait for "ready" status
5. Run analysis
6. Switch back to "Fake (Demo)"
7. Run analysis

**Expected:**

- No errors in console
- Each mode produces correct output format
- WASM worker terminates when switching away
- Memory is freed when switching away from WASM

---

### Test 2: Stop Functionality

**Objective:** Verify search can be stopped mid-execution

**Steps:**

1. Select WASM mode
2. Set depth to 20 (long search)
3. Click "Analyze"
4. After 2-3 depth updates, click "Stop"

**Expected:**

- Search stops immediately
- "Analysis stopped" appears in log
- Status returns to "ready"
- No errors in console

---

### Test 3: Multiple Sequential Analyses

**Objective:** Verify engine handles multiple requests correctly

**Steps:**

1. Select WASM mode
2. Run analysis (depth 8)
3. Wait for completion
4. Immediately run another analysis
5. Repeat 5 times

**Expected:**

- All analyses complete successfully
- No memory leaks
- Performance remains consistent
- Search count increases correctly

---

### Test 4: Performance Metrics

**Objective:** Verify performance monitoring works

**Steps:**

1. Select WASM mode
2. Click "Show Performance"
3. Run analysis
4. Observe metrics update in real-time
5. Click "Reset Metrics"
6. Run another analysis

**Expected:**

- WASM load time recorded on first use
- Search count increments
- Average search time updates
- Metrics reset to zero after clicking "Reset"
- Peak memory updates (Chrome/Edge only)

---

### Test 5: Error Handling

**Objective:** Verify graceful error handling

**Steps:**

1. Select WASM mode
2. Enter invalid FEN: `invalid fen string`
3. Click "Analyze"

**Expected:**

- Error message appears in log
- Status returns to "ready"
- Engine recovers and can run subsequent analyses

**Note:** Current scaffold implementation may not validate FEN

---

### Test 6: Browser Refresh

**Objective:** Verify state recovery after refresh

**Steps:**

1. Select WASM mode
2. Wait for "ready" status
3. Refresh page (F5 or Cmd+R)
4. Verify mode resets to default
5. Select WASM mode again
6. Run analysis

**Expected:**

- WASM reinitializes successfully
- No cached state causes issues
- Clean slate after refresh

---

## Compatibility Matrix

After testing, fill out this matrix:

| Browser        | Version | Platform    | WASM | Workers | SAB | Perf Mem | Status | Notes                 |
| -------------- | ------- | ----------- | ---- | ------- | --- | -------- | ------ | --------------------- |
| Chrome         | 120+    | Windows     | ✓    | ✓       | ✓   | ✓        | ✅     | Full support          |
| Chrome         | 120+    | macOS       | ✓    | ✓       | ✓   | ✓        | ✅     | Full support          |
| Chrome         | 120+    | Linux       | ✓    | ✓       | ✓   | ✓        | ✅     | Full support          |
| Firefox        | 121+    | Windows     | ✓    | ✓       | ✓   | ✗        | ✅     | No memory tracking    |
| Firefox        | 121+    | macOS       | ✓    | ✓       | ✓   | ✗        | ✅     | No memory tracking    |
| Firefox        | 121+    | Linux       | ✓    | ✓       | ✓   | ✗        | ✅     | No memory tracking    |
| Safari         | 17+     | macOS       | ✓    | ✓       | ✓   | ✗        | ✅     | No memory tracking    |
| Safari         | 17+     | iOS         | ✓    | ✓       | ✓   | ✗        | ⚠️     | Reduced performance   |
| Edge           | 120+    | Windows     | ✓    | ✓       | ✓   | ✓        | ✅     | Full support          |
| Edge           | 120+    | macOS       | ✓    | ✓       | ✓   | ✓        | ✅     | Full support          |
| Chrome Android | 120+    | Android 10+ | ✓    | ✓       | ✓   | ✓        | ✅     | Device-dependent perf |
| Firefox (old)  | <79     | Any         | ✓    | ✓       | ✗   | ✗        | ⚠️     | Slower WASM           |
| IE 11          | 11      | Windows     | ✗    | ✓       | ✗   | ✗        | ❌     | No WASM support       |
| Old Safari     | <11     | macOS/iOS   | ✗    | ✓       | ✗   | ✗        | ❌     | No WASM support       |

**Legend:**

- ✅ Full support
- ⚠️ Works with limitations
- ❌ Not supported
- WASM: WebAssembly
- SAB: SharedArrayBuffer
- Perf Mem: Performance Memory API

---

## Common Issues and Solutions

### Issue: "Engine not initialized"

**Symptoms:** Error message when trying to analyze

**Solutions:**

1. Wait for status to show "ready" before clicking Analyze
2. Check browser console for initialization errors
3. Verify WASM files are accessible at `/wasm/`
4. Try refreshing the page

---

### Issue: Status Stuck at "initializing"

**Symptoms:** WASM mode never reaches "ready"

**Solutions:**

1. Check browser console for errors
2. Verify WASM files exist:
   - `/wasm/engine_bridge_wasm.js`
   - `/wasm/engine_bridge_wasm_bg.wasm`
3. Check network tab for 404 errors
4. Try running `pnpm build:wasm` again
5. Clear browser cache and hard reload

---

### Issue: Slow Performance

**Symptoms:** Search takes much longer than expected

**Solutions:**

1. Check if browser is throttling background tabs
2. Verify you're not in private/incognito mode (may limit WASM)
3. Close other tabs/applications
4. Check Performance panel for bottlenecks
5. Try reducing depth to 6 for testing

---

### Issue: Compatibility Warnings

**Symptoms:** Yellow/red banner appears

**Solutions:**

1. Update browser to latest version
2. If on iOS Safari, reduced performance is expected (use fake mode for testing UI)
3. If SharedArrayBuffer warning, ignore (future feature)
4. If Performance Memory warning, ignore (Chrome/Edge only feature)

---

### Issue: No Search Info Updates

**Symptoms:** Only bestmove appears, no depth updates

**Solutions:**

1. Check browser console for worker communication errors
2. Verify worker is not blocked by CSP headers
3. Try refreshing and running again
4. Check if search depth is too low (depth 1 completes instantly)

---

## Performance Benchmarks

Use these benchmarks to compare browser performance:

### Benchmark 1: WASM Load Time

**Test:**

1. Clear browser cache
2. Refresh page
3. Switch to WASM mode
4. Note "WASM load time" in Performance panel

**Targets:**

- Desktop Chrome/Edge: <100ms
- Desktop Firefox: <150ms
- Desktop Safari: <150ms
- Mobile Chrome: <200ms
- Mobile Safari: <300ms

---

### Benchmark 2: Search Performance (Depth 8)

**Test:**

1. Use starting position FEN
2. Set depth to 8
3. Run analysis
4. Note total time from start to bestmove

**Targets:**

- Desktop Chrome/Edge: <100ms
- Desktop Firefox: <150ms
- Desktop Safari: <150ms
- Mobile Chrome: <200ms
- Mobile Safari: <300ms

**Note:** Scaffold implementation has minimal logic, so times should be fast

---

### Benchmark 3: Worker Creation

**Test:**

1. Check Performance panel after first WASM initialization
2. Note "Worker create time"

**Targets:**

- Desktop: <10ms
- Mobile: <20ms

---

## Reporting Test Results

When reporting test results, include:

1. **Browser Info:**
   - Name and version
   - Operating system
   - Device (for mobile)

2. **Test Results:**
   - Which tests passed/failed
   - Performance metrics
   - Any errors or warnings

3. **Console Output:**
   - Screenshot of browser console
   - Any errors or warnings

4. **Performance Data:**
   - WASM load time
   - Worker create time
   - Search times

5. **Screenshots:**
   - Compatibility banner (if present)
   - Performance panel
   - Any errors

---

## Automated Testing

While this guide focuses on manual testing, automated cross-browser tests can be run with:

```bash
# Run all tests
pnpm test

# Run specific browser detection tests
pnpm test browserDetect

# Run with UI (for debugging)
pnpm test --ui
```

**Note:** Automated tests run in Node/jsdom environment and may not catch browser-specific issues. Manual testing is still required.

---

## Next Steps

After completing cross-browser testing:

1. Update compatibility matrix with actual results
2. Document any browser-specific workarounds needed
3. File issues for any browser-specific bugs
4. Update documentation with browser recommendations
5. Consider adding automated Playwright/Cypress tests for critical flows

---

**Last Updated:** 2025-10-26
**Milestone:** M8 Session 9-10
**Status:** 🔄 In Progress
