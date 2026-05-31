export type NearbyDevice = {
  id: string;
  name: string;
  address: string;
  status: string;
  port?: number;
  lastSeen?: number;
};

export type TransportOption = {
  id: string;
  name: string;
  role: string;
  status: string;
  detail: string;
  action?: string | null;
  priority: number;
};

export type IncomingFile = {
  id: string;
  name: string;
  size: number;
  raw: string;
};

export type ActivityItem = {
  id: string;
  transferId?: string;
  title: string;
  detail: string;
  status: "waiting" | "active" | "done" | "failed";
  progress?: number;
  direction?: "send" | "receive" | "message";
  time: string;
};
