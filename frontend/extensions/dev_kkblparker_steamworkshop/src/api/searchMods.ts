import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/serialization/api-transform.ts';
import { totalResults, toWorkshopMods, workshopSearchResponseSchema } from '../schemas.ts';

export default async (uuid: string, params: { search?: string; sort?: string; page?: number }) => {
  const { data } = await axiosInstance.get(`/api/client/servers/${uuid}/workshop/mods`, {
    params: {
      search: params.search || undefined,
      sort: params.sort || undefined,
      page: params.page ?? 1,
    },
  });

  const result = parseFromApi(workshopSearchResponseSchema, data).result;

  return { mods: toWorkshopMods(result), total: totalResults(result) };
};
