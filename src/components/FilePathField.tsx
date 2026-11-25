import { FolderOpen } from 'lucide-react';
import { pickDirectory, pickFile } from '../api/dialogClient';
import styles from './FilePathField.module.css';

export interface FilePathFilter {
  name: string;
  extensions: string[];
}

export interface FilePathFieldProps {
  id: string;
  value: string;
  onChange: (value: string) => void;
  mode: 'file' | 'directory';
  dialogTitle?: string;
  filters?: FilePathFilter[];
  placeholder?: string;
  required?: boolean;
  disabled?: boolean;
  browseLabel?: string;
}

export function FilePathField({
  id,
  value,
  onChange,
  mode,
  dialogTitle,
  filters,
  placeholder,
  required,
  disabled,
  browseLabel = 'Procurar...',
}: FilePathFieldProps) {
  const handleBrowse = async () => {
    const selected =
      mode === 'directory'
        ? await pickDirectory({ title: dialogTitle, defaultPath: value || undefined })
        : await pickFile({ title: dialogTitle, filters, defaultPath: value || undefined });
    if (selected) {
      onChange(selected);
    }
  };

  return (
    <div className={styles.row}>
      <input
        id={id}
        className={styles.input}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        required={required}
        disabled={disabled}
      />
      <button
        type="button"
        className={styles.browseBtn}
        disabled={disabled}
        onClick={() => void handleBrowse()}
      >
        <FolderOpen size={15} aria-hidden="true" />
        <span>{browseLabel}</span>
      </button>
    </div>
  );
}
