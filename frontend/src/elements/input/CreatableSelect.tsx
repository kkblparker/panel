import { Combobox, InputBase, useCombobox } from '@mantine/core';
import { useEffect, useState } from 'react';
import { makeComponentHookable } from 'shared';
import { useTranslations } from '@/providers/TranslationProvider.tsx';

export interface CreatableSelectOption {
  value: string;
  label: string;
}

interface CreatableSelectProps {
  id?: string;
  data: CreatableSelectOption[];
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  disabled?: boolean;
  withAsterisk?: boolean;
}

// Like Select, but also accepts a value that isn't in `data` - typing something that doesn't
// match any option and confirming it (blur, Enter, or the "Use ..." option) submits the typed
// text verbatim as the value, with no matching label. Options still show label != value.
function CreatableSelect({ id, data, value, onChange, placeholder, disabled, withAsterisk }: CreatableSelectProps) {
  const { t } = useTranslations();
  const combobox = useCombobox({
    onDropdownClose: () => combobox.resetSelectedOption(),
  });

  const [search, setSearch] = useState(() => data.find((option) => option.value === value)?.label ?? value);

  useEffect(() => {
    setSearch(data.find((option) => option.value === value)?.label ?? value);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [value]);

  const trimmedSearch = search.trim();
  const filtered = data.filter((option) => option.label.toLowerCase().includes(trimmedSearch.toLowerCase()));
  const exactMatch = data.some(
    (option) => option.label === trimmedSearch || option.value === trimmedSearch,
  );

  const commitCustomValue = () => {
    if (trimmedSearch.length > 0 && !exactMatch) {
      onChange(trimmedSearch);
    } else if (trimmedSearch.length === 0) {
      onChange('');
    }
  };

  return (
    <Combobox
      store={combobox}
      withinPortal
      onOptionSubmit={(optionValue) => {
        if (optionValue === '$create') {
          onChange(trimmedSearch);
          combobox.closeDropdown();
          return;
        }

        const option = data.find((d) => d.value === optionValue);
        onChange(optionValue);
        setSearch(option?.label ?? optionValue);
        combobox.closeDropdown();
      }}
    >
      <Combobox.Target>
        <InputBase
          id={id}
          withAsterisk={withAsterisk}
          placeholder={placeholder}
          disabled={disabled}
          value={search}
          onChange={(e) => {
            setSearch(e.currentTarget.value);
            combobox.openDropdown();
            combobox.updateSelectedOptionIndex();
          }}
          onClick={() => combobox.openDropdown()}
          onFocus={() => combobox.openDropdown()}
          onBlur={() => {
            combobox.closeDropdown();
            commitCustomValue();
          }}
          rightSection={<Combobox.Chevron />}
          rightSectionPointerEvents='none'
        />
      </Combobox.Target>

      <Combobox.Dropdown>
        <Combobox.Options mah={250} style={{ overflowY: 'auto' }}>
          {filtered.map((option) => (
            <Combobox.Option value={option.value} key={option.value}>
              {option.label}
            </Combobox.Option>
          ))}
          {trimmedSearch.length > 0 && !exactMatch && (
            <Combobox.Option value='$create'>
              {t('elements.creatableSelectInput.useCustomValue', { value: trimmedSearch })}
            </Combobox.Option>
          )}
          {filtered.length === 0 && (trimmedSearch.length === 0 || exactMatch) && (
            <Combobox.Empty>{t('elements.selectInput.noResults', {})}</Combobox.Empty>
          )}
        </Combobox.Options>
      </Combobox.Dropdown>
    </Combobox>
  );
}

export default makeComponentHookable(CreatableSelect);
