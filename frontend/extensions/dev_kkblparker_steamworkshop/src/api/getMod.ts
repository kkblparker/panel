import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/serialization/api-transform.ts';
import { toWorkshopMods, WorkshopMod, workshopModResponseSchema } from '../schemas.ts';

export default async (uuid: string, modId: string): Promise<WorkshopMod | null> => {
  const { data } = await axiosInstance.get(`/api/client/servers/${uuid}/workshop/mods/${modId}`);
  const result = parseFromApi(workshopModResponseSchema, data).result;

  return toWorkshopMods(result)[0] ?? null;
};
