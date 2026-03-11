const { invoke } = window.__TAURI__.core;

export async function searchMemories(query, limit, activityType, appName) {
  return await invoke('search_memories', {
    query,
    limit,
    activityType,
    appName
  });
}

export async function getTimeline(days) {
  return await invoke('get_timeline', { days });
}

export async function getMemoryGraph() {
  return await invoke('get_memory_graph');
}

export async function togglePause() {
  return await invoke('toggle_pause');
}

export async function getStatus() {
  return await invoke('get_status');
}

export async function getSettings() {
  return await invoke('get_settings');
}

export async function updateSettings(newSettings) {
  return await invoke('update_settings', { newSettings });
}

export async function deleteMemory(activityId) {
  return await invoke('delete_memory', { activityId });
}

export async function getDebugMetrics() {
  return await invoke('get_debug_metrics');
}

export async function previewActivity(activityId) {
  return await invoke('preview_activity', { activityId });
}
