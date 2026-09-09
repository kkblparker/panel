import { faDownload, faTrash } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { Badge, Image, Text } from '@mantine/core';
import debounce from 'debounce';
import { useEffect, useMemo, useState } from 'react';
import { httpErrorToHuman } from '@/api/axios.ts';
import Button from '@/elements/buttons/Button.tsx';
import ServerContentContainer from '@/elements/containers/ServerContentContainer.tsx';
import Card from '@/elements/data-display/Card.tsx';
import Spinner from '@/elements/feedback/Spinner.tsx';
import Select from '@/elements/input/Select.tsx';
import Group from '@/elements/layout/Group.tsx';
import { useServerCan } from '@/plugins/usePermissions.ts';
import { useToast } from '@/providers/ToastProvider.tsx';
import { useServerStore } from '@/stores/server.ts';
import getInstalledMods from './api/getInstalledMods.ts';
import installMod from './api/installMod.ts';
import searchMods from './api/searchMods.ts';
import uninstallMod from './api/uninstallMod.ts';
import { WorkshopMod } from './schemas.ts';

const RESULTS_PER_PAGE = 20;

const SORT_OPTIONS = [
  { value: 'popularity', label: 'Popularity' },
  { value: 'newest', label: 'Newest' },
  { value: 'recently-updated', label: 'Recently Updated' },
  { value: 'highest-rated', label: 'Highest Rated' },
  { value: 'subscribers', label: 'Subscribers' },
  { value: 'name', label: 'Name' },
];

export default function ServerWorkshop() {
  const { addToast } = useToast();
  const server = useServerStore((state) => state.server);
  const canManage = useServerCan('workshop.manage');

  const [searchInput, setSearchInput] = useState('');
  const [search, setSearch] = useState('');
  const [sort, setSort] = useState<string | null>('popularity');
  const [page, setPage] = useState(1);

  const [mods, setMods] = useState<WorkshopMod[]>([]);
  const [hasNextPage, setHasNextPage] = useState(false);
  const [loading, setLoading] = useState(true);
  const [installedModIds, setInstalledModIds] = useState<Set<string>>(new Set());
  const [pendingModId, setPendingModId] = useState<string | null>(null);

  useEffect(() => {
    getInstalledMods(server.uuid)
      .then((installed) => setInstalledModIds(new Set(installed.map((mod) => mod.modId))))
      .catch((error) => addToast(httpErrorToHuman(error), 'error'));
  }, []);

  useEffect(() => {
    setLoading(true);

    searchMods(server.uuid, { search: search || undefined, sort: sort || undefined, page })
      .then((result) => {
        const data = result.data ?? [];
        setMods(data);
        setHasNextPage(data.length >= RESULTS_PER_PAGE);
      })
      .catch((error) => addToast(httpErrorToHuman(error), 'error'))
      .finally(() => setLoading(false));
  }, [search, sort, page]);

  const setDebouncedSearch = useMemo(
    () =>
      debounce((value: string) => {
        setPage(1);
        setSearch(value);
      }, 400),
    [],
  );

  const doInstall = (mod: WorkshopMod) => {
    setPendingModId(mod.id);
    installMod(server.uuid, mod.id, { name: mod.name, version: mod.version })
      .then(() => {
        addToast(`${mod.name} has been installed.`, 'success');
        setInstalledModIds((prev) => new Set(prev).add(mod.id));
      })
      .catch((error) => addToast(httpErrorToHuman(error), 'error'))
      .finally(() => setPendingModId(null));
  };

  const doUninstall = (mod: WorkshopMod) => {
    setPendingModId(mod.id);
    uninstallMod(server.uuid, mod.id)
      .then(() => {
        addToast(`${mod.name} has been removed.`, 'success');
        setInstalledModIds((prev) => {
          const next = new Set(prev);
          next.delete(mod.id);
          return next;
        });
      })
      .catch((error) => addToast(httpErrorToHuman(error), 'error'))
      .finally(() => setPendingModId(null));
  };

  return (
    <ServerContentContainer
      title='Workshop'
      search={searchInput}
      setSearch={(value) => {
        const resolved = typeof value === 'function' ? value(searchInput) : value;
        setSearchInput(resolved);
        setDebouncedSearch(resolved);
      }}
      contentRight={
        <Select
          data={SORT_OPTIONS}
          value={sort}
          onChange={(value) => {
            setPage(1);
            setSort(value);
          }}
          w={200}
        />
      }
    >
      {loading ? (
        <Spinner.Centered />
      ) : mods.length === 0 ? (
        <Text c='dimmed'>No workshop mods found.</Text>
      ) : (
        <div className='grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4'>
          {mods.map((mod) => {
            const installed = installedModIds.has(mod.id);

            return (
              <Card key={mod.id} p='md'>
                {mod.imageUrl && <Image src={mod.imageUrl} height={140} radius='sm' mb='sm' alt={mod.name} />}
                <Text fw={600}>{mod.name}</Text>
                {mod.author && (
                  <Text size='xs' c='dimmed'>
                    by {mod.author}
                  </Text>
                )}
                {mod.summary && (
                  <Text size='sm' mt='xs' lineClamp={3}>
                    {mod.summary}
                  </Text>
                )}
                {mod.tags && mod.tags.length > 0 && (
                  <Group gap='xs' mt='sm'>
                    {mod.tags.slice(0, 3).map((tag) => (
                      <Badge key={tag} size='sm' variant='light'>
                        {tag}
                      </Badge>
                    ))}
                  </Group>
                )}
                <Group justify='space-between' mt='md'>
                  <Text size='xs' c='dimmed'>
                    {mod.sizeFormatted ?? ''}
                  </Text>
                  {installed ? (
                    <Button
                      size='xs'
                      color='red'
                      variant='light'
                      leftSection={<FontAwesomeIcon icon={faTrash} />}
                      disabled={!canManage}
                      loading={pendingModId === mod.id}
                      onClick={() => doUninstall(mod)}
                    >
                      Uninstall
                    </Button>
                  ) : (
                    <Button
                      size='xs'
                      color='blue'
                      leftSection={<FontAwesomeIcon icon={faDownload} />}
                      disabled={!canManage}
                      loading={pendingModId === mod.id}
                      onClick={() => doInstall(mod)}
                    >
                      Install
                    </Button>
                  )}
                </Group>
              </Card>
            );
          })}
        </div>
      )}

      <Group justify='center' mt='lg'>
        <Button variant='subtle' disabled={page <= 1} onClick={() => setPage((p) => p - 1)}>
          Previous
        </Button>
        <Text size='sm'>{page}</Text>
        <Button variant='subtle' disabled={!hasNextPage} onClick={() => setPage((p) => p + 1)}>
          Next
        </Button>
      </Group>
    </ServerContentContainer>
  );
}
