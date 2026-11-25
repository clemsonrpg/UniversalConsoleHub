# Universal Trophy System — Universal Console Hub 3.0

## Regra Absoluta de Integridade & Proveniência
Toda conquista/troféu possui uma **origem imutável** (`origin`):
- `official` — Exclusivamente proveniente de API oficial autorizada (nenhuma ativa nas Fases 1–2).
- `emulator` — Lida somente leitura de arquivos locais de emulador (ex.: RPCS3 `TROPCONF.SFM` + `TROPUSR.DAT` — **Fase 4**, acoplada às sessões reais do PS3 da Fase 3) ou provedor de emulação autorizado (ex.: RetroAchievements com API key do usuário — **Fase 6**).
- `local` — Conquistas locais próprias do Universal Console Hub (`LocalAchievementProvider`).

### Proteções Técnicas Implementadas na Fase 1 (Preservadas em Todas as Fases)
1. **Imutabilidade de Origem no SQLite:** Triggers `BEFORE UPDATE` (`trg_trophies_immutable_origin` e `trg_trophy_unlocks_immutable_origin`) abortam qualquer tentativa de alterar `origin` após a inserção.
2. **Prevenção de Duplicatas:**
   - `trophies`: `UNIQUE(game_id, provider_id, external_trophy_id)`
   - `trophy_unlocks`: `UNIQUE(profile_id, trophy_id)`
3. **Providers Arquiteturais:** `PlayStationTrophyProvider`, `XboxAchievementProvider`, `SteamAchievementProvider`, `EmulatorAchievementProvider`, `LocalAchievementProvider` cadastrados com detecção honesta de disponibilidade. Apenas `LocalAchievementProvider` está ativo na base das Fases 1–2; a leitura real de troféus do RPCS3 acoplada às sessões entra na **Fase 4** (Opção A), e integrações externas autorizadas (RetroAchievements / Steam Web API com chave do usuário) entram na **Fase 6**.
