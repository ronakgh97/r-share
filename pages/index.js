const API_URL = 'http://140.245.17.34:8080/api/relay/status';
const REFRESH_INTERVAL = 3000;
const FETCH_TIMEOUT = 5000;

let intervalId = null;
let isPageVisible = true;

// Utility functions
function formatUptime(seconds) {
    if (!Number.isFinite(seconds)) return '—';

    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = Math.floor(seconds % 60);

    if (days > 0) {
        return `${days}d ${hours}h ${minutes}m`;
    } else if (hours > 0) {
        return `${hours}h ${minutes}m ${secs}s`;
    } else if (minutes > 0) {
        return `${minutes}m ${secs}s`;
    } else {
        return `${secs}s`;
    }
}

function formatBytes(gb) {
    if (!Number.isFinite(gb)) return '—';

    if (gb >= 1) {
        return `${gb.toFixed(2)} GB`;
    } else {
        return `${(gb * 1024).toFixed(2)} MB`;
    }
}

function formatSpeed(mbps) {
    if (!Number.isFinite(mbps)) return '—';
    return `${mbps.toFixed(2)} MB/s`;
}

function formatTimestamp(isoString) {
    if (!isoString) return '—';

    try {
        const date = new Date(isoString);
        return date.toLocaleTimeString();
    } catch {
        return '—';
    }
}

function formatMemory(usedMB, maxMB) {
    if (!Number.isFinite(usedMB) || !Number.isFinite(maxMB)) return '—';

    const usedGB = (usedMB / 1024).toFixed(2);
    const maxGB = (maxMB / 1024).toFixed(2);
    return `${usedGB} / ${maxGB} GB`;
}

function safeSetText(id, value) {
    const el = document.getElementById(id);
    if (el) el.textContent = value;
}

function safeSetWidth(id, percent) {
    const el = document.getElementById(id);
    if (el) el.style.width = `${Math.min(100, Math.max(0, percent))}%`;
}

function updateDashboard(data) {
    safeSetText('serverVersion', data.serverVersion || '—');
    safeSetText('uptime', formatUptime(data.uptimeSeconds));
    safeSetText('totalBandwidth', formatBytes(data.totalBandwidthGB));
    safeSetText('activeSessions', data.activeSessions ?? '—');
    safeSetText('pendingSessions', data.pendingSessions ?? '—');
    safeSetText('completedSessions', data.totalSessionsCompleted ?? '—');
    safeSetText('failedSessions', data.totalSessionsFailed ?? '—');
    safeSetText('avgSpeed', formatSpeed(data.averageTransferSpeedMBps));
    safeSetText('peakBandwidth', formatSpeed(data.peakBandwidthMBps));
    safeSetText('currentTransfers', data.currentTransferCount ?? '—');
    safeSetText('threadCount', data.threadCount ?? '—');
    safeSetText('timestamp', formatTimestamp(data.timestamp));

    const memoryPercent = (data.memoryUsedMB / data.memoryMaxMB) * 100;
    safeSetText('memoryUsage', formatMemory(data.memoryUsedMB, data.memoryMaxMB));
    safeSetWidth('memoryBar', memoryPercent);

    const cpuPercent = data.cpuUsagePercent;
    safeSetText('cpuUsage', Number.isFinite(cpuPercent) ? `${cpuPercent.toFixed(1)}%` : '—');
    safeSetWidth('cpuBar', cpuPercent);

    const statusDot = document.getElementById('statusDot');
    const statusText = document.getElementById('statusText');
    if (statusDot && statusText) {
        statusDot.className = 'status-dot online';
        statusText.textContent = 'Online';
    }
}

function setOfflineStatus() {
    const statusDot = document.getElementById('statusDot');
    const statusText = document.getElementById('statusText');
    if (statusDot && statusText) {
        statusDot.className = 'status-dot offline';
        statusText.textContent = 'Offline';
    }
}

async function fetchStatus() {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), FETCH_TIMEOUT);

    try {
        const response = await fetch(API_URL, {
            signal: controller.signal,
            cache: 'no-store'
        });

        clearTimeout(timeoutId);

        if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
        }

        const contentType = response.headers.get('content-type') || '';
        if (!contentType.includes('application/json')) {
            throw new Error('Invalid content type');
        }

        const data = await response.json();
        updateDashboard(data);
    } catch (error) {
        clearTimeout(timeoutId);

        if (error.name === 'AbortError') {
            console.error('Fetch timeout:', error);
        } else {
            console.error('Error fetching status:', error);
        }

        setOfflineStatus();
    }
}

function startPolling() {
    stopPolling();
    fetchStatus();
    intervalId = setInterval(() => {
        if (isPageVisible) {
            fetchStatus();
        }
    }, REFRESH_INTERVAL);
}

function stopPolling() {
    if (intervalId !== null) {
        clearInterval(intervalId);
        intervalId = null;
    }
}

document.addEventListener('visibilitychange', () => {
    isPageVisible = !document.hidden;
    if (isPageVisible) {
        fetchStatus();
    }
});

window.addEventListener('beforeunload', () => {
    stopPolling();
});

startPolling();
