import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/serialization/api-transform.ts';
import { toWorkshopMods, WorkshopMod, workshopInstalledResponseSchema } from '../schemas.ts';

export default async (uuid: string): Promise<WorkshopMod[]> => {
  const { data } = await axiosInstance.get(`/api/client/servers/${uuid}/workshop/installed`);
  const result = parseFromApi(workshopInstalledResponseSchema, data).result;

  return toWorkshopMods(result);
};
