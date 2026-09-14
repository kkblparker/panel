import { axiosInstance } from '@/api/axios.ts';

export default async (uuid: string, value: string): Promise<void> => {
  await axiosInstance.put(`/api/client/servers/${uuid}/steam-workshop/load-order`, { value });
};
