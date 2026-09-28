import {
  getCurrentWindow,
  currentMonitor,
  LogicalSize,
  LogicalPosition,
} from "@tauri-apps/api/window";
import { native } from "./api";
export type Mode = "bottom" | "right" | "floating" | "collapsed";
export async function arrange(mode: Mode, expanded: boolean) {
  if (!native) return;
  const win = getCurrentWindow();
  const monitor = await currentMonitor();
  if (!monitor) return;
  const area = monitor.workArea;
  const ratio = monitor.scaleFactor;
  const x = area.position.x / ratio,
    y = area.position.y / ratio;
  const width = area.size.width / ratio,
    height = area.size.height / ratio;
  let w = Math.min(1100, width),
    h = Math.min(expanded ? 730 : 330, height);
  let left = x + (width - w) / 2,
    top = y + height - h;
  if (mode === "right") {
    w = expanded ? Math.min(760, width) : Math.min(390, width);
    h = height;
    left = x + width - w;
    top = y;
  }
  if (mode === "floating") {
    w = Math.min(1100, width);
    h = Math.min(expanded ? 730 : 400, height);
    top = y + (height - h) / 2;
  }
  if (mode === "collapsed") {
    w = Math.min(520, width);
    h = 120;
    left = x + width - w;
    top = y + height - h;
  }
  await win.setSize(new LogicalSize(w, h));
  await win.setPosition(new LogicalPosition(left, top));
}
export async function setPinned(value: boolean) {
  if (native) await getCurrentWindow().setAlwaysOnTop(value);
}
export async function minimize() {
  if (native) await getCurrentWindow().minimize();
}
