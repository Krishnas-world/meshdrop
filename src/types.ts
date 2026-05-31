export type NearbyDevice = {
  name: string;
  address: string;
  status: string;
};

export type IncomingFile = {
  id: string;
  name: string;
  size: number;
  raw: string;
};

export type ActivityItem = {
  id: string;
  title: string;
  detail: string;
  status: "waiting" | "done" | "failed";
  time: string;
};
