import {
  AlertTriangle,
  Ban,
  Cpu,
  FlaskConical,
  Gamepad2,
  Monitor,
} from 'lucide-react';
import type { CapabilityState } from '../types/domain';
import styles from './CapabilityBadge.module.css';

export interface CapabilityBadgeProps {
  state: CapabilityState;
}

const CAPABILITY_META: Record<
  CapabilityState,
  { label: string; title: string; className: string; Icon: typeof Cpu }
> = {
  FULL_SYSTEM_BOOT: {
    label: 'Boot Completo',
    title: 'FULL_SYSTEM_BOOT: suporta inicialização do software de sistema com arquivos do usuário.',
    className: styles.full_system_boot ?? '',
    Icon: Cpu,
  },
  ORIGINAL_DASHBOARD: {
    label: 'Dashboard Original',
    title: 'ORIGINAL_DASHBOARD: expõe a interface original do console via emulação real.',
    className: styles.original_dashboard ?? '',
    Icon: Monitor,
  },
  PARTIAL_DASHBOARD: {
    label: 'Dashboard Parcial',
    title: 'PARTIAL_DASHBOARD: interface original parcialmente disponível com ressalvas.',
    className: styles.partial_dashboard ?? '',
    Icon: AlertTriangle,
  },
  DIRECT_GAME_ONLY: {
    label: 'Jogo Direto',
    title: 'DIRECT_GAME_ONLY: execução direta de títulos suportada.',
    className: styles.direct_game_only ?? '',
    Icon: Gamepad2,
  },
  EXPERIMENTAL: {
    label: 'Experimental',
    title: 'EXPERIMENTAL: suporte upstream em desenvolvimento ativo; estabilidade varia por título.',
    className: styles.experimental ?? '',
    Icon: FlaskConical,
  },
  UNSUPPORTED: {
    label: 'Não Suportado',
    title: 'UNSUPPORTED: recurso indisponível neste emulador.',
    className: styles.unsupported ?? '',
    Icon: Ban,
  },
};

export function CapabilityBadge({ state }: CapabilityBadgeProps) {
  const { label, title, className, Icon } = CAPABILITY_META[state];
  return (
    <span className={`${styles.badge} ${className}`} title={title} data-capability={state}>
      <Icon size={13} aria-hidden="true" />
      <span>{label}</span>
    </span>
  );
}
