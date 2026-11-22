# Verificações locais — 0.2.0 — 2026-11-22

Ambiente: Windows 11 x64, Rust MSVC 1.98.1, Slint 1.19.0, Python 3.9.

- Rustfmt e Clippy com warnings como erros: aprovados.
- 14 testes unitários: aprovados (core, plataforma, áudio, executor, updater e armazenamento).
- UI test: valores de preferências preservados ao criar a página.
- Smoke real: anel visível, NOACTIVATE, TOPMOST, centro exato de 360 px e foreground preservado; aprovado com backend padrão e com SLINT_BACKEND=winit-software. O cenário inicial exibia também o editor e teve interferência de sua ativação; corrigido para testar somente o overlay.
- Probe: três botões instalaram/removeram hooks sem interceptar; leitura Core Audio e Potato Strip 0/7, Bus 0/4 bem-sucedida, sem alterar valores.
- Build otimizado Windows x64: aprovado.
- Empacotador MSI extrai o CAB e exige hashes idênticos aos arquivos de origem.

A inspeção visual final foi interrompida pelo usuário com Esc. A interface já havia sido inspecionada durante o desenvolvimento; a última compilação não recebeu nova inspeção visual completa. Não foram executados comandos de desligamento/reinicialização nem instalação/upgrade/rollback do MSI no computador pessoal. Essas verificações continuam no roteiro manual, junto a DPI misto, jogos, elevação e acessibilidade.
