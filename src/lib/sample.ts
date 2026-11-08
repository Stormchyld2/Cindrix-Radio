import type { Track } from "./api";
// Placeholder rows until the library scanner lands (milestone 3).
export const sampleTracks: Track[] = [
  { id: "s1", title: "North Fork", subtitle: "Highland Road", kind: "Music", duration_secs: 198 },
  { id: "s2", title: "Copper Hollow", subtitle: "Wren Valley", kind: "Music", duration_secs: 224 },
  { id: "s3", title: "Slow Burn", subtitle: "Maple Static", kind: "Music", duration_secs: 280 },
  { id: "s4", title: "Weather & News", subtitle: "Radio · stream", kind: "Radio", duration_secs: null },
];
export const sampleDrives = [
  { name: "Music C:", count: "5,912 items", online: true },
  { name: "Media Drive D:", count: "6,934 items", online: true },
  { name: "USB-Backup E:", count: "1,204 unavailable", online: false },
];
