# Verificações locais — 0.2.1 — 2026-11-22

Ambiente: Windows 11 x64, Rust MSVC 1.98.1, Slint 1.19.0.

- 16 testes unitários, Rustfmt e Clippy com warnings como erros.
- Regressão reproduzida no controlador: um clique curto processado com atraso podia fechar o anel. O teste agora usa os instantes reais de pressionar/soltar, percorre cinco botões em modo clique/segurar e mantém a janela aberta por múltiplos eventos antes de verificar visibilidade/foco.
- A configuração de janela não podia usar o título para distinguir o anel: o hook do Slint recebe o nome genérico do processo nessa fase. A identificação passou para o momento de construção do componente.
- Removido o monitor assíncrono de foreground; o controlador usa o foreground atual a cada frame, enquanto o hook deixa de interceptar imediatamente quando a origem não é mais foreground.
- Teste de interface verifica cor/opacidade carregadas; snapshot do renderizador confirmou RGB (70,69,69) e alpha 94 para #464545 com 37% de opacidade. Imagens de teste ficam somente em target.
- JSON legado sem background_color continua válido. Teste de armazenamento verifica persistência da cor e opacidade.
- Cinco códigos de botão são testados com seus pares pressionar/soltar. Botões específicos do fabricante além dos cinco padrões dependem de mapeamento pelo driver.

A validação do controlador não injeta mouse e não instala hooks ativos. Testes físicos prolongados com o mouse do usuário e instalação/rollback do MSI em VM continuam necessários. O roteiro geral está em TEST_PLAN.md.

---

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
