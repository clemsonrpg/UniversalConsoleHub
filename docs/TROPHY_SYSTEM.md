# Universal Trophy System — Universal Console Hub 3.0

## Regra Absoluta de Integridade & Proveniência
Toda conquista/troféu possui uma **origem imutável** (`origin`):
- `official` — Exclusivamente proveniente de API oficial autorizada (nenhuma ativa na Fase 1).
- `emulator` — Lida de arquivos locais de emulador (ex.: RPCS3 `TROPUSR.DAT` — Fase 3) ou provedor de emulação autorizado (ex.: RetroAchievements com API key do usuário — Fase 6).
- `local` — Conquistas locais próprias do Universal Console Hub (`LocalAchievementProvider`).

### Proteções Técnicas Implementadas na Fase 1
1. **Imutabilidade de Origem no SQLite:** Triggers `BEFORE UPDATE` (`trg_trophies_immutable_origin` e `trg_trophy_unlocks_immutable_origin`) abortam qualquer tentativa de alterar `origin` após a inserção.
2. **Prevenção de Duplicatas:**
   - `trophies`: `UNIQUE(game_id, provider_id, external_trophy_id)`
   - `trophy_unlocks`: `UNIQUE(profile_id, trophy_id)`
3. **Providers Arquiteturais (Seção 13):** `PlayStationTrophyProvider`, `XboxAchievementProvider`, `SteamAchievementProvider`, `EmulatorAchievementProvider`, `LocalAchievementProvider` cadastrados com detecção honesta de disponibilidade. Apenas `LocalAchievementProvider` está pronto na Fase 1; os demais reportam indisponibilidade transparente até suas respectivas fases/APIs autorizadas.
