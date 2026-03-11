export function formatTimestamp(timestamp) {
  const date = new Date(timestamp * 1000);
  return date.toLocaleTimeString('en-US', {
    hour: '2-digit',
    minute: '2-digit'
  });
}

export function formatDate(timestamp) {
  const date = new Date(timestamp * 1000);
  const today = new Date();
  const yesterday = new Date(today);
  yesterday.setDate(yesterday.getDate() - 1);
  
  if (date.toDateString() === today.toDateString()) {
    return 'Today';
  } else if (date.toDateString() === yesterday.toDateString()) {
    return 'Yesterday';
  } else {
    return date.toLocaleDateString('en-US', {
      weekday: 'long',
      month: 'long',
      day: 'numeric'
    });
  }
}

export function formatRelativeTime(timestamp) {
  const now = Date.now();
  const time = timestamp * 1000;
  const diff = now - time;
  
  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);
  
  if (minutes < 1) return 'Just now';
  if (minutes < 60) return `${minutes}m ago`;
  if (hours < 24) return `${hours}h ago`;
  if (days < 7) return `${days}d ago`;
  
  return formatDate(timestamp);
}

export function truncateText(text, maxLength = 150) {
  if (!text || text.length <= maxLength) return text;
  return text.substring(0, maxLength).trim() + '...';
}

export function formatNumber(num) {
  if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M';
  if (num >= 1000) return (num / 1000).toFixed(1) + 'K';
  return num.toString();
}

export function formatActivityType(type) {
  const types = {
    'app_focus': 'Application',
    'clipboard': 'Clipboard',
    'screenshot': 'Screenshot',
    'ocr_text': 'OCR Text',
    'file': 'File',
    'url': 'URL'
  };
  return types[type] || type;
}

export function getActivityIcon(type) {
  const icons = {
    'app_focus': '⌨️',
    'clipboard': '📋',
    'screenshot': '📸',
    'ocr_text': '📝',
    'file': '📄',
    'url': '🌐'
  };
  return icons[type] || '📌';
}
