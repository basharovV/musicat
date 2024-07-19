export function timeSince(timestamp) {
    const now = Date.now();
    const secondsPast = Math.floor((now - timestamp) / 1000);
    if (secondsPast < 60) {
        return "just now";
    }
    if (secondsPast < 3600) {
        const minutes = Math.floor(secondsPast / 60);
        return minutes + (minutes === 1 ? " minute ago" : " minutes ago");
    }
    if (secondsPast < 86400) {
        const hours = Math.floor(secondsPast / 3600);
        return hours + (hours === 1 ? " hour ago" : " hours ago");
    }
    if (secondsPast < 604800) {
        const days = Math.floor(secondsPast / 86400);
        return days + (days === 1 ? " day ago" : " days ago");
    }
    if (secondsPast < 2592000) {
        const weeks = Math.floor(secondsPast / 604800);
        return weeks + (weeks === 1 ? " week ago" : " weeks ago");
    }
    if (secondsPast < 31536000) {
        const months = Math.floor(secondsPast / 2592000);
        return months + (months === 1 ? " month ago" : " months ago");
    }
    const years = Math.floor(secondsPast / 31536000);
    return years + (years === 1 ? " year ago" : " years ago");
}
