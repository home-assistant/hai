import { openUrl } from "@tauri-apps/plugin-opener";

export async function openExternalUrl(url: string) {
  try {
    await openUrl(url);
  } catch {
    // Fallback for development/web
    window.open(url, "_blank");
  }
}

export async function openExternalLink(event: Event, url: string) {
  event.preventDefault();
  await openExternalUrl(url);
}
