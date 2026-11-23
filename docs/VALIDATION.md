# Verificações locais — 0.2.12 — 2026-11-23

- 27 testes unitários passaram; formatação e Clippy sem avisos.
- `--ui-test` verificou bloqueio de entrada por seleção implícita e soltura no submenu configurado para clique, clique com acionamento pelo botão esquerdo, confirmação por Enter e preservação do modo anterior.
- A opção por ação foi desmarcada/marcada, salva, recarregada ao trocar de posição e serializada. JSON antigo sem a opção mantém o comportamento anterior.
- Captura real do editor em `target/submenu-click-test.png` inspecionada: seletor de destino, checkbox e explicação visíveis.
- Teste de controlador usa eventos sem instalar hooks nem injetar entrada. Uso prolongado com mouse físico e instalação/upgrade em VM continuam pendentes.

---

# Verificações locais e CI — 0.2.11 — 2026-11-23

- CI Windows aprovado: formatação, Clippy, 26 testes Rust, um teste Python de RTF, build release, MSI e publicação automática.
- Execução: https://github.com/whoslucaxs/orbit-axion/actions/runs/45225110341
- Release v0.2.11 publicada por `github-actions[bot]`, com cinco assets e alvo no commit `cd73325cd5928116dafedc31eda5b8b579ce3396`.
- EXE, MSI e checksums baixados da release; SHA-256 confere com os digests do GitHub e SHA256SUMS. MSI publicado passou na validação ICE; executável extraído é idêntico ao EXE publicado. Licença e notices extraídos conferem com os digests dos assets correspondentes.
- Metadados verificaram português (1046), sete diálogos padrão, navegação para escopo e pasta, propriedades para instalação por usuário ou máquina e preservação do UpgradeCode. Build local também passou usando .NET 8.
- Extração do ícone pelo Windows verificada nos tamanhos grande e pequeno. A prévia de diagnóstico não foi instalada.
- O ambiente bloqueou o controle visual de `msiexec.exe`; o assistente não foi percorrido automaticamente. Instalação/upgrade/rollback em VM continuam pendentes.

---

# Verificações locais e CI — 0.2.10 — 2026-11-23

- CI Windows aprovado: formatação, Clippy, 26 testes, build release, geração e verificação do MSI e publicação automática.
- Execução: https://github.com/whoslucaxs/orbit-axion/actions/runs/45211913693
- Release v0.2.10 criada por `github-actions[bot]`, com cinco assets publicados e alvo no commit validado `bb91849d1a268b84de9005641e41f4bb5ed923b6`.
- EXE e MSI baixados da release; SHA-256 confere com o arquivo de checksums e os digests do GitHub.
- Caminho de versão existente exercitado com a v0.2.9: arquivos publicados preservados.
- Recuperação de draft interrompido e instalação/upgrade/rollback do MSI em VM não foram exercitados nesta rodada.

---

# Verificações locais — 0.2.9 — 2026-11-23

- PNG original preservado byte a byte, com transparência. ICO contém 16, 20, 24, 32, 40, 48, 64, 128 e 256 pixels.
- Recursos PE do executável lidos via APIs Windows: grupo de ícones e os nove recursos de imagem presentes, com tamanhos corretos.
- UI test verificou a imagem incorporada de 256px; captura do editor com a nova marca inspecionada.
- Regressões de ícones, filtros, submenu e acionamento passaram; Clippy e formatador sem avisos.
- MSI inclui ícone da entrada de aplicativos instalados e do atalho. Instalação/upgrade/rollback em VM continuam pendentes.

---

# Verificações locais — 0.2.8 — 2026-11-23

- 26 testes unitários passaram; Clippy e formatador sem avisos.
- UI test usa o mesmo callback da seleção do destino para importar automaticamente o ícone real de um executável.
- Verificados indicador de extração, aplicação à prévia sem sair da ação, filtros ajustáveis e persistência após salvar e trocar de botão.
- Regressões anteriores de submenu e acionamento passaram.
- Instalação/upgrade/rollback do MSI em VM continuam pendentes.

---

# Verificações locais — 0.2.7 — 2026-11-23

- 26 testes unitários: filtros preservam alpha e cores sem ajustes, escala de cinza, brilho, contraste, inversão, limites e compatibilidade com configurações antigas.
- UI test extraiu o ícone real de System32/notepad.exe no worker, verificou preto e branco inicial, mudança da prévia, imagem de origem intacta e persistência dos filtros após trocar de botão.
- Caminho com separadores mistos normalizado para a API Shell; COM inicializado e liberado no contexto da extração.
- Captura da página de filtros inspecionada; controles e botões visíveis.
- Regressões de submenu e acionamento passaram; formatador e Clippy sem avisos.
- Instalação/upgrade/rollback do MSI em VM continuam pendentes.

---

# Verificações locais — 0.2.6 — 2026-11-23

- 24 testes unitários passaram; catálogo inclui os dois identificadores de aplicativos.
- UI test verificou SVGs incorporados de 256px, categoria Aplicativos, busca sem distinção de maiúsculas, seleção, prévia e gravação dos dois ícones em configuração isolada.
- Captura da biblioteca inspecionada com os dois ícones renderizados.
- Regressões de submenu e acionamento passaram; formatador e Clippy sem avisos.
- Instalação/upgrade/rollback do MSI em VM continuam pendentes.

---

# Verificações locais — 0.2.5 — 2026-11-22

- 24 testes unitários: catálogo de ambas as bibliotecas, cores individuais, geometria central, compatibilidade JSON e verificações anteriores.
- Regressão de submenu em janela real: estado de botão mantido após transição e menu fechado na soltura sobre centro, vazio e alvo de submenu; modo clique preservado.
- UI test aplica cores de botão/ícone, seleciona ícone Bootstrap e salva o resultado, sem executar a ação.
- Capturas do renderizador inspecionadas para Bootstrap e Material, seletor visual, círculo central e efeito de vidro; testes com backend padrão e software.
- Teste de alpha do fundo detecta clamp incorreto de opacidade; cantos continuam transparentes e fundo nativo desativado.
- Fontes incorporadas verificadas no código gerado pelo Slint (registro por memória).

O seletor nativo ChooseColorW foi aberto em diagnóstico isolado, mostrou a paleta visual e aceitou RGB (255,172,172); a confirmação retornou #FFACAC, validando o fluxo real e a conversão de canais. Nenhuma configuração do usuário foi alterada por esse diagnóstico. O efeito Liquid Glass é visual, sem APIs Apple de refração. Instalação/upgrade/rollback do MSI em VM e testes prolongados em outros dispositivos continuam pendentes.

---

# Verificações locais — 0.2.4 — 2026-11-22

- 22 testes unitários: migração de cores, independência após reinício, validação RGB/RGBA e verificações anteriores.
- UI test seleciona F8 e F13 pela lista e verifica o código real escolhido.
- UI test salva a aparência do menu e exige que as demais preferências permaneçam idênticas.
- Comparação do buffer completo do menu antes/depois de alterar fundo, opacidade, destaque e tema do aplicativo.
- Snapshot da página Cores do menu em target/menu-colors-test.png; cantos do anel continuam transparentes.

A lista de teclas funciona sem armar os hooks. A validação não substitui testes de teclas proprietárias/reservadas em todo hardware.

---

# Verificações locais — 0.2.3 — 2026-11-22

- 18 testes unitários incluem tecla configurada, compatibilidade do JSON, persistência, pares de entrada, repetição e soltura após fechamento.
- UI test carrega F13 como acionamento e percorre os cinco botões do mouse mais teclas em modo clique/segurar, em janela real sem interceptação.
- Probe instala/remove hooks desarmados para mouse, F13 e F24.
- Entradas sintetizadas pelo executor recebem marcador para evitar reativação pelo próprio Orbit.
- Mantidos os testes de fundo nativo desativado, cantos transparentes, foco e geometria da 0.2.2.

A captura usa uma tecla individual, sem combinação. No editor a tecla de ativação fica livre para digitação. Atalhos reservados pelo sistema e variações de hardware/driver não foram validados em todos os ambientes.

---

# Verificações locais — 0.2.2 — 2026-11-22

- Desativado o backdrop DWM que preenchia os limites retangulares da janela do anel.
- Smoke consulta DWMWA_SYSTEMBACKDROP_TYPE na janela real e exige DWMSBT_NONE quando o atributo está disponível.
- Snapshot do anel verifica alpha zero nos quatro cantos; cor e opacidade continuam aplicadas ao círculo.
- Removida a linha decorativa que extrapolava a borda circular.
- Permanecem as verificações de foco, geometria, cinco botões e persistência.

O teste consulta o estado nativo e o buffer do renderizador; a aparência final em outros drivers/compositores exige validação nesses ambientes.

---

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
