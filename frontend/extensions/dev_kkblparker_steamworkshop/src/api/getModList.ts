import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/serialization/api-transform.ts';
import { toWorkshopMods, WorkshopMod, workshopInstalledResponseSchema } from '../schemas.ts';

export default async (
  uuid: string,
): Promise<{ mods: WorkshopMod[]; loadOrderSupported: boolean; loadOrder: string | null }> => {
  const { data } = await axiosInstance.get(`/api/client/servers/${uuid}/workshop/installed`);
  const parsed = parseFromApi(workshopInstalledResponseSchema, data);

  return {
    mods: toWorkshopMods(parsed.result),
    loadOrderSupported: parsed.load_order_supported,
    loadOrder: parsed.load_order,
  };
};
