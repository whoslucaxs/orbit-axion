# Actions Ring para Windows — Especificação de Produto e Arquitetura

> **Nome do Projeto:** Orbit Axion
> **Status:** requisitos aprovados em conversa; documento de planejamento, não implementação.
> **Versão do documento:** 1.0 — 22/11/2026  
> **Plataforma-alvo:** Windows 11 (x64 inicialmente)  
> **Stack:** Rust + Slint + APIs nativas do Windows (Win32/COM)  
> **Licença do projeto:** GNU GPL v3  
> **Distribuição:** repositório GitHub, sem aceitar contribuições externas inicialmente; releases com instalador MSI.

## 1. Visão do produto

Aplicativo de menu radial inspirado no conceito de Actions Ring, independente de periféricos Logitech. Permite disparar ações, macros e controles de mídia com um botão convencional do mouse. Possui oito posições fixas por menu, níveis ilimitados de submenus, perfis automáticos por aplicativo e interface de configuração visual. A aplicação roda em segundo plano, com foco em baixo consumo de recursos e funcionamento offline.

**Extensão de escopo solicitada após a definição das 30 features:** compatibilidade com **Voicemeeter Potato** e ajuste do volume com a **roda de rolagem do mouse enquanto o cursor estiver sobre um ícone de volume**. Essa extensão é tratada como **Feature 31 / requisito transversal de áudio**, integrada às Features 03, 04, 07, 12, 18 e 25.

### Objetivos

- Abrir menu radial na posição exata do cursor usando exclusivamente um botão do mouse configurável.
- Permitir personalização integral das oito ações, seus ícones, nomes e submenus.
- Manter o foco na janela anterior e encaminhar atalhos/texto à aplicação correta.
- Suportar ações do Windows e controles de volume, incluindo canais e buses do Voicemeeter Potato.
- Oferecer editor visual, perfis por executável, backup, recuperação e instalador MSI.
- Não usar Electron, telemetria, serviços de nuvem nem sistema de plugins externos.

### Não objetivos (versão inicial)

- macOS, Linux ou versão portátil.
- Login, conta online ou sincronização entre computadores.
- Driver próprio, injeção em processos, bypass de anticheat ou garantia em tela cheia exclusiva.
- Gravador de macros, loops, condições de macros, ícones animados, vibração tátil.
- Histórico de uso, marketplace de plugins ou traduções externas.

## 2. Requisitos funcionais aprovados — Features 01 a 30

### F01 — Ativação
- **Exclusivamente por botão do mouse**; não criar atalho global de teclado para abertura.
- Botão de ativação configurável; menu aparece na localização atual do cursor.

### F02 — Estrutura radial
- **8 posições fixas** em cada menu.
- Cada posição pode conter uma ação, um submenu ou estar vazia.
- Submenus permitidos; nomes e ícones personalizados.

### F03 — Tipos de ação
- Abrir aplicativos.
- Enviar atalhos de teclado.
- Executar comandos (incluindo programas e argumentos).
- Abrir sites, arquivos e pastas.
- Controlar mídia.
- Controlar janelas.
- Inserir textos pré-configurados.
- Ações do sistema (bloquear, suspender, desligar e reiniciar, com opções de confirmação).
- **Complemento posterior:** ações de áudio Voicemeeter Potato e volume por scroll (F31).

### F04 — Interação
- Dois modos: **clique** para abrir/selecionar e **segurar, mover, soltar** para escolher.
- Destaque da ação sob cursor.
- ESC e clique fora fecham o menu.
- Voltar ao centro cancela seleção no modo de gesto; em submenus, o centro permite retornar ao nível anterior (F13).

### F05 — Perfis por aplicativo
- Seleção automática do perfil conforme aplicativo em primeiro plano.
- Perfil global como fallback.
- Troca manual de perfis permitida.
- Espaços não configurados podem herdar ações do perfil global.

### F06 — Personalização visual
- Temas claro e escuro seguindo a configuração do Windows 11.
- Fundo com **acrílico, transparência e desfoque**.
- Normalmente apenas ícones; nome surge ao selecionar/passar o cursor.
- Animações suaves de abertura/fechamento/seleção.
- Cores, tamanho do menu e transparência ajustáveis.

### F07 — Editor visual
- Interface gráfica completa para perfis, menus, ações e macros.
- Drag-and-drop para reorganizar as oito posições.
- Prévia do menu atualizada em tempo real.
- Importação/exportação de configurações e perfis.
- Backup automático.

### F08 — Integração com Windows
- Opção de iniciar junto com o Windows.
- Ícone e comandos na bandeja do sistema.
- Instância única.
- **Sem versão portátil**.
- Permitir sobreposição em tela cheia **quando tecnicamente compatível**.
- Verificação automática de atualizações com notificações.

### F09 — Botões de mouse
- Suportar apenas **botão do meio (MButton), Mouse 4 (XBUTTON1), Mouse 5 (XBUTTON2)**.
- Captura automática do botão durante a configuração.
- Suprimir sua função original quando usado para ativar o menu, na medida em que o sistema permitir.
- Sem configuração individual por mouse físico.
- Limiar de tempo configurável para distinguir clique de pressionar/segurar.

### F10 — Macros simples
- Sequências ordenadas de ações.
- Atrasos configuráveis entre passos.
- Cancelamento durante execução.
- Editor visual de etapas.
- Sem gravador, lógica condicional ou loops.

### F11 — Desempenho e privacidade
- Renderização sob demanda.
- Aceleração por GPU quando disponível.
- Carregamento de recursos sob demanda.
- Logs técnicos **locais**, sem telemetria e sem registrar histórico de ações executadas.

### F12 — Ações dinâmicas básicas
- Ação de mídia alterna **reproduzir/pausar** conforme estado.
- Ação de janela alterna **maximizar/restaurar** conforme estado.
- Sem contexto da área de transferência ou regras personalizadas.

### F13 — Submenus
- Profundidade ilimitada em dados, limitada na prática por proteção de memória/validação de ciclos.
- Abertura **por clique**, não por hover.
- Centro do círculo retorna ao submenu anterior.
- Transições animadas.
- No modo de pressionar/soltar, soltar sobre entrada de submenu **abre o submenu sem executar ação final**; gesto posterior requer nova ativação para evitar cliques acidentais.

### F14 — Ícones
- Biblioteca integrada de ícones.
- Importação de **PNG, JPG e ICO** (não SVG/WebP na importação inicial).
- Extração de ícones de executáveis/atalhos Windows.
- Alteração de cores dos ícones vetoriais compatíveis da biblioteca.
- Sem ícones animados.

### F15 — Posicionamento e monitores
- Menu **centrado exatamente no cursor**, mesmo que fique parcialmente cortado pela borda. **Não** reposicionar nem reduzir automaticamente.
- Suportar múltiplos monitores; escolher o monitor sob o cursor.
- Suportar DPI por monitor.
- Não memorizar tamanho por monitor.

### F16 — Segurança e autorização
- Confirmação habilitável **individualmente por ação**.
- Solicitar elevação via UAC quando usuário configurar execução elevada; processo principal não precisa rodar como administrador.
- Executar scripts `.ps1`, `.bat`, `.cmd` explicitamente configurados.
- Validar caminhos, argumentos e configuração; alertar sobre comandos potencialmente destrutivos **sem prometer detecção infalível**.
- Sem histórico de ações.

### F17 — Busca de ações
- Pesquisa abrange todos os submenus do **perfil ativo**.
- Digitação inicia a pesquisa com o menu aberto.
- Busca aproximada/fuzzy.
- Setas navegam nos resultados, Enter executa.
- Abertura do menu continua exclusiva pelo mouse.

### F18 — Gestos direcionais
- Oito direções: cima, cima-direita, direita, baixo-direita, baixo, baixo-esquerda, esquerda, cima-esquerda.
- Distância mínima configurável.
- Feedback visual da direção em seleção.
- Gesto utiliza o mesmo mapeamento de oito posições do menu, sem sequências complexas.

### F19 — Feedback sonoro
- Sons opcionais para abertura, fechamento, mudança de seleção, sucesso e erro.
- Volume dos efeitos independente do volume principal.
- Sem importação de sons personalizados e sem feedback háptico.

### F20 — Ferramentas avançadas
- Monitor de RAM, CPU e latência de abertura/tempo de resposta.
- Visualizador de logs técnicos.
- Restaurar configurações padrão.
- Listar e restaurar backups.
- Testar eventos dos botões do mouse.
- Exportar diagnóstico manualmente; sem envio automático.
- Sem modo de depuração detalhado separado.

### F21 — Persistência
- **JSON**, sem SQLite.
- Sem sincronização.
- Migração automática de esquemas de configuração.
- Verificação de integridade com backup recuperável.
- **Sem requisito de isolamento privado por usuário**; ainda respeitar regras normais de permissões do Windows e não gravar configurações em diretórios do sistema sem autorização.

### F22 — Extensibilidade
- **Sem sistema de plugins**.
- Organização interna do código em módulos Rust é decisão técnica, não API de plugins.

### F23 — Instalação e distribuição
- Instalador **MSI**.
- Escolha de instalação: somente usuário atual ou todos os usuários (com elevação quando necessária).
- Atualizações verificadas automaticamente (F08), mas aplicadas apenas após aprovação do usuário.
- Atalho no menu Iniciar.
- Desinstalador permite manter ou apagar configurações.
- **Assinatura digital de código não planejada**; documentar possíveis avisos de reputação/SmartScreen e nunca recomendar contorná-los como requisito.

### F24 — Idiomas
- Português brasileiro (`pt-BR`) e inglês (`en`).
- Detectar idioma de exibição do Windows na primeira execução.
- Troca de idioma nas configurações, sem reinstalação.
- Sem importação de traduções externas JSON.

### F25 — Foco e ciclo de vida do menu
- Overlay não deve ativar sua própria janela e deve preservar o foco do aplicativo original.
- **Fechar overlay antes de executar** a ação.
- Fechar ao detectar troca de janela em primeiro plano.
- Overlay topmost para janelas normais, respeitando limitações do Windows.
- Busca recebe texto via mecanismo de captura controlado somente enquanto o menu estiver ativo; impedir propagação indevida de teclas quando necessário. Investigar interações com IME/acessibilidade.

### F26 — Interface técnica
- **Slint + Rust**.
- Usar **APIs nativas do Windows** para hooks, foco, overlay, acrílico, tray, janelas, áudio e instalação/integridade quando necessário.
- Separar janela do editor e overlay do menu.

### F27 — Gerenciamento dos perfis
- Associação de perfil **apenas selecionando executável manualmente**, não por lista de processos em execução.
- Duplicar perfis.
- Associar vários executáveis ao mesmo perfil.
- Prioridade configurável quando regras coincidirem.
- Sem funcionalidade separada de substituição temporária do perfil automático; preservar opção de troca manual de F05 com comportamento definido em UX.

### F28 — Jogos
- Configuração de disponibilidade **por jogo/executável**.
- Suprimir o botão original apenas quando a ativação for possível.
- Tentar overlay em tela cheia exclusiva via mecanismos normais do Windows, sem garantia de funcionamento.
- Desativar captura nos aplicativos comprovadamente incompatíveis e notificar.
- Não injetar DLL, modificar processos protegidos ou contornar sistemas anticheat.

### F29 — Recuperação
- Restaurar automaticamente o **último backup válido** após corrupção detectada; registrar e notificar.
- Reinicialização controlada após falha, com limite de tentativas e backoff.
- Modo de segurança que não instala hooks de mouse.
- Rollback de atualização quando houver instalação falha, com mecanismos compatíveis com MSI.
- Notificar recuperações realizadas.

### F30 — Licença e comunidade
- Licença **GNU GPL v3** para o código do projeto.
- Código hospedado no GitHub.
- **Não** aceitar contribuições externas inicialmente (documentar política de PRs).
- Publicar código e instaladores MSI no GitHub Releases.
- Confirmar licenças de todas as dependências antes de cada publicação. Slint oferece uso GPLv3 para projetos open source; SDK oficial do Voicemeeter permite desenvolver integrações, mas **não redistribuir DLLs ou outros binários proprietários sem revisar termos específicos**.

## 3. Extensão F31 — Áudio, Voicemeeter Potato e scroll no ícone de volume

### 3.1 Requisito essencial

**Com o menu radial aberto e o cursor sobre um ícone configurado como controle de volume, girar a roda do mouse para cima deve aumentar e para baixo diminuir o volume do alvo configurado.** O ajuste deve ocorrer sem clique e sem fechar o menu. O overlay deve atualizar seu feedback visual em tempo real.

A funcionalidade suporta dois provedores:

1. **Áudio Windows:** volume principal do dispositivo de saída selecionado, via Windows Core Audio; opcionalmente volumes de sessão/aplicativo quando a ação permitir selecionar uma sessão concreta.
2. **Voicemeeter Potato:** ganho (`gain`, em dB) de **strips** de entrada e **buses** de saída escolhidos na configuração da ação, através da **Voicemeeter Remote API oficial**.

### 3.2 Alvos de volume configuráveis

- **Windows:** volume master do endpoint de áudio escolhido (incluindo dispositivo padrão), com nível exibido em `%`.
- **Potato:** `Strip[0..7].Gain` e `Bus[0..4].Gain`, conforme os índices da instância Potato, exibidos em `dB`. Os canais são identificados no editor também por nome/label obtido na integração sempre que disponível.
- **Sem dedução automática de que o ícone controla o volume geral**: a ação armazena `provider`, `target_type`, `target_id`, passo, limites e política de mudo.
- Exemplo: um ícone «Microfone» controla `Strip[0].Gain`; «Discord» pode controlar um strip virtual que o usuário designar; «Saída A1» controla `Bus[0].Gain` (o rótulo final será confirmado pelo mapeamento real do Voicemeeter).

### 3.3 Interação e concorrência com gestos

- Scroll só é interceptado para controlar volume **enquanto o cursor está sobre a região ativa do ícone de volume** e o menu está aberto.
- Scroll sobre outros ícones não modifica volume. Política padrão: ignorar rolagem dentro do overlay fora do ícone de volume; não repassar involuntariamente à janela em segundo plano.
- Scroll para cima = aumentar; para baixo = diminuir.
- **Scroll não executa a ação do ícone, não troca submenu e não fecha o menu.** Um clique no ícone pode executar a ação principal configurada, como `mute/unmute` (definir no editor); o scroll sempre ajusta o volume.
- A região ativa deve acompanhar o ícone durante animações e mudanças de submenu.
- Prioridade de eventos: se o menu estiver no modo segurar/soltar, o scroll sobre o ícone ajusta o volume sem disparar a ação; soltar o botão após rolar **não deve executar a ação involuntariamente**. Implementar estado explícito `wheel_adjusted` para consumir o gesto de soltura, salvo configuração futura deliberada.
- Ao fechar o menu, remover interceptação temporária do wheel, liberar recursos e preservar comportamento normal de scroll do Windows.
- Caso o Voicemeeter não esteja instalado, aberto ou acessível, a ação não altera outro alvo silenciosamente: mostrar estado «Indisponível», manter o menu utilizável e registrar falha técnica.

### 3.4 Passos, limites e exibição

- **Passo de scroll configurável** por ação: valor inicial sugerido **2% por notch** no Windows e **1 dB por notch** no Potato (sugestão de implementação, não decisão adicional do usuário).
- Permitir agregar rodas de alta resolução / trackpads sem amplificação inesperada, com normalização por delta e limitação de taxa de atualizações.
- Respeitar limites retornados pela API/controlador para cada parâmetro; ao atingir limite, não ultrapassar.
- Exibir volume atual ao selecionar/rolar: `%` para Windows, `dB` para Potato; mostrar `MUTE` quando aplicável.
- Evitar sobrescrever mudanças externas: ler valor real mais recente antes de aplicar delta; sincronizar estado se o usuário movimentar o fader no Voicemeeter.
- Scroll deve ter baixa latência, não bloquear a thread de renderização e tratar desconexão/reconexão da Remote API.

### 3.5 Integração técnica do Voicemeeter

- Criar módulo Rust `audio/voicemeeter.rs` isolado dos controles genéricos do Windows.
- Usar SDK / Remote API fornecida pela VB-Audio, com `Login`, `Logout`, leitura e escrita de parâmetros por funções oficiais (por exemplo `GetParameterFloat` / `SetParameterFloat`).
- Carregar biblioteca e símbolos a partir de instalação válida do Voicemeeter ou conforme documentação oficial. **Não incluir DLL proprietária no MSI sem autorização verificável.**
- Detectar versão/edição e habilitar alvos Potato somente quando compatíveis.
- Não assumir que o Voicemeeter está rodando: mostrar estado e opção de tentar reconectar.
- Implementar cache leve de labels e estado; não acumular hooks ou sessões de API ao reabrir o overlay.
- Expor interface abstrata `VolumeProvider` para Windows e Potato, sem plugins externos.

### 3.6 Critérios de aceitação específicos

1. Menu aberto, ponteiro sobre ícone configurado `Potato: Strip 0`: um notch para cima muda o ganho configurado (exemplo inicial +1 dB) e atualiza valor visual.
2. Um notch para baixo reduz o mesmo canal.
3. Ícone `Windows Master`: scroll altera o volume do endpoint escolhido e mostra percentual.
4. Scroll fora de ícones de volume nunca altera o volume.
5. O menu permanece aberto durante a rolagem.
6. Pressionar/segurar/rolar/soltar não dispara clique acidental.
7. Mudanças externas no Voicemeeter são refletidas na próxima leitura.
8. Sem Voicemeeter instalado: ícones correspondentes indicam indisponibilidade; os demais controles seguem funcionais.
9. O scroll fora do overlay volta ao comportamento normal imediatamente após fechamento.
10. Mudança de monitor, escala DPI e tema não quebram a região de hover do volume.

## 4. Resolução de interações entre features

| Interação | Regra de produto |
|---|---|
| F01 + F09 | Um dos três botões convencionais configurado abre o overlay; teclas não abrem o menu. |
| F04 + F13 | Clique abre submenu; no modo segurar/soltar, soltar sobre submenu navega, não executa uma ação terminal. |
| F04 + F18 | Reconhecimento de gesto usa 8 setores com limiar de distância; voltar ao centro cancela a seleção. |
| F04 + F31 | Scroll sobre volume tem precedência e não gera execução na soltura do botão. |
| F05 + F27 | Perfil automático usa executável e prioridade; fallback global preenche ações não definidas. |
| F06 + F15 | Não deslocar overlay nas bordas, apesar de itens ficarem cortados. |
| F17 + F25 | Capturar texto só durante pesquisa ativa; não roubar foco nem digitar involuntariamente na janela de origem. |
| F08 + F23 + F29 | Verificação notifica, instalação MSI exige aprovação e mecanismo de recuperação fica disponível. |
| F11 + F20 | Logs técnicos/diagnóstico permitidos; histórico de uso pessoal proibido. |
| F16 + F03 | Executar comandos é poder explícito do usuário; validação/confirmadores não constituem sandbox de segurança. |
| F28 + F09 | Não consumir entrada do mouse em jogo se não for possível abrir overlay. |

## 5. Arquitetura técnica recomendada

```text
                       +----------------------------+
                       |  Windows 11 / Mouse / DPI  |
                       +-------------+--------------+
                                     |
                             Win32 Input Adapter
                       (botões, wheel, foreground)
                                     |
                                     v
 +----------------------+   +--------+----------+    +------------------------+
 | Slint Settings UI    |<->| App Controller     |<-->| Profiles & JSON Store  |
 | perfis/editor/import |   | Event / State Mach.|    | migrations & backup    |
 +----------------------+   +---+------------+---+    +------------------------+
                                |            |
                     +----------v--+      +--v--------------------------+
                     | Radial Menu |      | Action Executor             |
                     | Slint UI    |      | Win32/COM/PowerShell/macros |
                     +-------------+      +---+-------------------------+
                                              |
                                     +--------v---------+
                                     | Audio Controller |
                                     | Windows CoreAudio|
                                     | Voicemeeter API  |
                                     +------------------+
```

### 5.1 Processo, threads e mensagens

- Um executável principal, instância única, com tray + janela de configurações + overlay radial.
- Thread de UI Slint dedicada a desenho/eventos; não executar scripts, I/O ou chamadas potencialmente lentas da API de áudio nessa thread.
- Hooks de mouse e monitoramento de janela em componente Win32 com loop apropriado; despachar eventos por canal interno para o controlador.
- Executor de ações/macro em worker separado com cancelamento cooperativo e timeout configurável.
- Controlador de áudio serializa alterações por alvo, usa leituras recentes, reintenta conexão quando apropriado.
- Armazenamento com escrita atômica: temporário, flush, rename/substituição compatível e backup prévio.
- Ao encerrar, soltar hooks, API Voicemeeter, interfaces COM, tray, workers e janelas.

### 5.2 Interfaces sugeridas (conceituais, não código compilável)

```rust
trait ActionExecutor {
    fn execute(&self, action: Action, context: ExecutionContext) -> ActionResult;
}

trait VolumeProvider {
    fn available(&self) -> bool;
    fn read_level(&self, target: &VolumeTarget) -> Result<VolumeLevel, AudioError>;
    fn adjust_level(&self, target: &VolumeTarget, delta: VolumeDelta)
        -> Result<VolumeLevel, AudioError>;
}

enum MenuPhase {
    Closed,
    OpenIdle,
    HoldingButton,
    SubmenuTransition,
    Searching,
    AdjustingVolume,
}
```

**Observação:** a implementação real deve adotar assinaturas async/thread-safe adequadas, callbacks Win32 e regras de afinidade de thread Slint/COM.

### 5.3 APIs do Windows a avaliar

- `WH_MOUSE_LL` / `SetWindowsHookEx` para mouse global convencional e supressão do botão quando aplicável; validar também limitações de desktop elevado e integridade de processos.
- `GetForegroundWindow`, `GetWindowThreadProcessId`, consultas de executável e eventos de foreground para perfis.
- `WS_EX_NOACTIVATE`, `WS_EX_TOPMOST` e configuração de janela para overlay sem captura indevida do foco.
- DWM / composição para transparência acrílica; avaliar APIs disponíveis e fallback de material se indisponível.
- `GetDpiForWindow`, `MonitorFromPoint` e `GetMonitorInfo` para DPI/monitor.
- Windows Core Audio (`IAudioEndpointVolume` e, quando necessário, sessões de áudio) para volume.
- Notificações Windows, registro de inicialização/Startup e MSI para ciclo de vida.
- UAC por execução elevada explícita; não elevar o processo residente por padrão.

> **Atenção:** a captura de roda via `WH_MOUSE_LL`, a transparência/perfil de janela e a busca sem foco precisam de protótipos específicos. As APIs possuem restrições e diferenças entre aplicativos comuns, elevados e tela cheia. Não prometer compatibilidade absoluta.

## 6. Modelo de dados JSON (esboço)

```json
{
  "schema_version": 1,
  "general": {
    "language": "system",
    "theme": "system",
    "start_with_windows": true,
    "mouse_button": "xbutton1",
    "hold_threshold_ms": 220,
    "gesture_min_distance_px": 48,
    "sound_enabled": false
  },
  "profile_order": ["global", "development"],
  "profiles": [
    {
      "id": "global",
      "name": "Global",
      "priority": 0,
      "executables": [],
      "slots": [
        {
          "id": "slot-0",
          "kind": "volume",
          "label": "Volume Windows",
          "icon": {"source": "library", "name": "volume-2"},
          "volume": {
            "provider": "windows",
            "target_type": "master",
            "target_id": "default",
            "wheel_step": 2.0,
            "wheel_unit": "percent",
            "on_click": "toggle_mute"
          }
        },
        null, null, null, null, null, null, null
      ]
    },
    {
      "id": "development",
      "name": "Desenvolvimento",
      "priority": 10,
      "executables": ["C:\\Program Files\\Microsoft VS Code\\Code.exe"],
      "inherit_global": true,
      "slots": [
        {
          "id": "slot-0",
          "kind": "volume",
          "label": "Voicemeeter Mic",
          "icon": {"source": "library", "name": "mic"},
          "volume": {
            "provider": "voicemeeter_potato",
            "target_type": "strip",
            "target_id": 0,
            "wheel_step": 1.0,
            "wheel_unit": "db",
            "on_click": "toggle_mute"
          }
        },
        null, null, null, null, null, null, null
      ]
    }
  ]
}
```

Valores acima ilustram **formato e padrões sugeridos**, não representam decisões adicionais aprovadas. `slots` deve ter sempre oito entradas e submenus não podem criar ciclos. Dados sensíveis em comandos e textos são mantidos locais em JSON; avisar na UI que o armazenamento não é criptografado.

### 6.1 Organização de arquivos de dados

```text
<app-data-directory>/ActionsRing/
  config.json
  profiles.json
  icons/
  backups/
    2026-11-22T180000Z.zip
  logs/
  diagnostics/
```

O caminho definitivo será escolhido respeitando escopo MSI e permissões do Windows, sem impor isolamento privado por usuário como requisito funcional. Armazenar diretórios relativos nos manifests sempre que possível.

## 7. Estrutura de repositório proposta

```text
actions-ring/
├── Cargo.toml
├── Cargo.lock
├── LICENSE                        # GNU GPL-3.0
├── README.md
├── CONTRIBUTING.md                 # contribuições externas não aceitas inicialmente
├── CHANGELOG.md
├── docs/
│   ├── SPEC.md
│   ├── ARCHITECTURE.md
│   ├── AUDIO_VOICEMEETER.md
│   ├── SECURITY.md
│   └── TEST_PLAN.md
├── assets/
│   ├── icons/
│   ├── sounds/
│   └── locales/
├── crates/
│   ├── app/                        # entrypoint, lifecycle, tray, controller
│   │   └── src/
│   ├── core/                       # modelos, actions, perfis, state machine
│   │   └── src/
│   ├── windows-platform/           # Win32 hooks, DPI, foreground, overlay
│   │   └── src/
│   ├── audio/                      # Windows Core Audio + Voicemeeter Remote API
│   │   └── src/
│   ├── storage/                    # JSON, migração, backup, integridade
│   │   └── src/
│   └── updater/                    # verificação, MSI, rollback e recuperação
│       └── src/
├── ui/
│   ├── radial-menu.slint
│   ├── settings.slint
│   ├── editor.slint
│   ├── profiles.slint
│   ├── audio-target-picker.slint
│   └── components/
├── packaging/
│   └── windows-msi/
├── tests/
│   ├── integration/
│   └── fixtures/
└── .github/
    └── workflows/
        ├── ci.yml
        └── release.yml
```

Separação em crates é recomendação de engenharia: **não** implica sistema de plugins.

## 8. Roadmap de implementação

### Etapa 0 — Pesquisa técnica e protótipos de risco
- Criar workspace Rust e protótipo Slint.
- Prototipar overlay não ativável, topmost, transparente/acrílico e com DPI por monitor.
- Testar mouse MButton/XBUTTON1/XBUTTON2, supressão condicional, clique/hold e wheel.
- Prototipar busca sem roubo de foco.
- Prototipar ligação com `VoicemeeterRemote.dll`: leitura/escrita do gain de strip/bus, reconexão e direitos de distribuição.
- Decidir solução de MSI/rollback com testes reais de instalação por usuário e máquina.
- **Saída:** relatório de viabilidade e riscos, antes de criar grande parte da UI.

### Etapa 1 — MVP de interação (F01, F02, F04, F09, F15, F25, F26)
- Aplicativo residente com tray e instância única.
- Menu de oito setores em posição exata do cursor, sem reposicionamento nas bordas.
- Clique, hold/soltar, destaque, ESC, clique fora e retorno ao centro.
- Identificação do foreground e execução mantendo/restaurando foco conforme contrato.
- **Aceite:** menu abre e fecha confiavelmente, sem consumir clique/scroll fora do uso pretendido.

### Etapa 2 — Ações fundamentais e áudio por scroll (F03, F12, F31)
- Executor de apps, atalhos, comandos, arquivos, mídia, janelas, texto e sistema.
- Core Audio para volume Windows.
- Voicemeeter Potato Remote API para strips/buses; editor básico de alvo e passo.
- Hover no ícone de volume + wheel, feedback, saturação, indisponibilidade e testes de hold/soltar.
- **Aceite:** scroll sobre ícone controla alvo correto, sem fechar menu ou enviar rolagem por engano.

### Etapa 3 — Submenus e perfis (F05, F13, F18, F27, F28)
- Submenus recursivos e navegação pelo centro.
- Perfis por executável, prioridades, fallback e herança global.
- Gestos de oito direções, distância mínima, feedback.
- Disponibilidade por jogo e comportamento seguro nos incompatíveis.

### Etapa 4 — Editor, personalização e pesquisa (F06, F07, F14, F17, F24)
- Editor Slint drag-and-drop com prévia ao vivo.
- Ícones integrados, importação PNG/JPG/ICO e extração de executáveis.
- Configuração do acrílico, cores, tema, tamanho e transparência.
- Busca fuzzy, navegação por teclado e i18n pt-BR/en.

### Etapa 5 — Macros, validação e observabilidade (F10, F11, F16, F19, F20)
- Editor de macros sequenciais, delays e cancelamento.
- Confirmação por ação, scripts, UAC solicitado, validação de caminhos/comandos.
- Sounds opcionais e painel de desempenho.
- Logs locais, teste de mouse e exportação manual de diagnóstico.

### Etapa 6 — Persistência, robustez e distribuição (F08, F21, F22, F23, F29, F30)
- JSON com versionamento de esquema, migração, atomic write e backup.
- Recuperação de backup, safe mode e watchdog limitado.
- MSI por usuário/por máquina, desinstalação com preservação opcional.
- GitHub Actions CI, releases, updater com consentimento e recuperação de falhas.
- Auditoria de licenças GPLv3/Slint/Voicemeeter e terceiros.

### Etapa 7 — Qualidade e release 1.0
- Testes de integração, regressão UI, DPI, múltiplos monitores, hooks, foreground e jogos compatíveis.
- Testes de carga/idle de CPU/RAM e latência de abertura.
- Testes de scroll com wheel de alta resolução e volumes Windows/Voicemeeter reais.
- Testes de migração/restore, MSI, rollback, atualização, uninstall e tradução.
- README, screenshots, licença, notas de release e binário MSI no GitHub Releases.

## 9. Plano de testes mínimos

### Interação
- Cada botão suportado abre overlay e, quando configurado, não executa ação original.
- Hold curto/long press e click distinguíveis sob diferentes taxas de eventos.
- 8 setores e diagonais funcionam com DPI 100%, 125%, 150%, 200%.
- Clique fora, ESC, mudança de janela e centro cancelam/navegam corretamente.
- Foco original preservado; atalhos atingem a aplicação de origem.

### Configuração
- Oito slots fixos em todos os menus.
- Importar/exportar JSON, migrar esquemas, validar IDs, sem ciclos em submenus.
- Múltiplos executáveis por perfil, prioridade e herança funcionam.
- Backup restaurado recupera perfis e ícones; corrupção não causa perda silenciosa.

### Áudio e Voicemeeter
- Master Windows pelo scroll no ícone.
- Strip 0, strip 7, bus 0, bus 4 do Potato: leitura/alteração de ganho.
- Limites, volume com mute, reconexão após reiniciar Voicemeeter.
- Scroll sobre não-volume e com menu fechado nunca altera ganho.
- Scroll enquanto botão de ativação está pressionado não executa ação em soltar.
- Garantir fechamento/liberação da API após encerramento.

### Instalação
- MSI por usuário e para máquina; UAC quando requerido.
- Desinstalação mantém/apaga configs conforme escolha.
- Atualização exigindo aprovação do usuário, rollback após instalação falha.
- Publicação com código completo e avisos de terceiros.

## 10. Riscos e decisões pendentes de engenharia (não são novas features)

1. **Overlay vs tela cheia exclusiva:** sobreposição pode falhar. Aceitar degradação graciosa e regras por executável.
2. **Low-level hook e aplicativos elevados:** diferentes níveis de integridade e processos protegidos podem impedir captura/supressão.
3. **Foco sem ativação vs texto de busca:** implementar captura de entrada cuidadosamente; não quebrar digitação normal/IME.
4. **Acabamento acrílico no Slint:** pode exigir janela custom Win32 + composição DWM, conforme versão do framework.
5. **Scroll em wheel de alta resolução:** normalização e coalescing são fundamentais para não gerar saltos bruscos.
6. **Voicemeeter API:** validação de índices, limites, versionamento, disponibilidade e redistribuição de DLLs.
7. **MSI + atualização/reversão:** projetar backup do instalador anterior e mecanismo de rollback verificável; evitar autoelevação silenciosa.
8. **JSON não criptografado:** comandos e textos potencialmente sensíveis ficam legíveis; comunicar risco no editor.
9. **Licenças:** Slint GPLv3 adequado ao projeto GPLv3; rever licenças de crates, ícones, sons e materiais do Voicemeeter antes de publicar.
10. **Requisitos conflitantes resolvidos:** posicionamento exato no cursor prevalece sobre evitar corte; ausência de plugins prevalece sobre qualquer extensão externa; captura de scroll apenas para ícone de volume prevalece sobre propagação do wheel.

## 11. Definição de pronto (DoD)

- Compila em pipeline reproduzível para Windows 11.
- Todos os critérios de aceitação de Features 01–31 foram testados ou tiveram limitação documentada.
- Usuário configura perfis e oito slots sem editar JSON manualmente.
- Menu responde sem alterar indevidamente foco, scroll ou botão em aplicativos externos.
- Voicemeeter Potato e áudio Windows ajustam volume por scroll ao passar no ícone de volume.
- Sem telemetria, sem histórico de ações, sem plugins, sem sincronização.
- MSI instala, atualiza, remove e recupera configurações conforme escopo.
- Publicação GPLv3 inclui código correspondente, avisos de terceiros e changelog.

## 12. Referências oficiais

- Slint Licensing: https://slint.dev/faqs e https://github.com/slint-ui/slint/blob/master/LICENSE.md
- Voicemeeter SDK oficial: https://github.com/vburel2018/Voicemeeter-SDK
- Documentação Voicemeeter Remote API: https://download.vb-audio.com/Download_CABLE/VoicemeeterRemoteAPI.pdf
- VB-Audio Licensing: https://vb-audio.com/Services/licensing.htm
- Voicemeeter Potato: https://vb-audio.com/Voicemeeter/potato.htm

---

**Nota sobre fidelidade ao escopo:** esta especificação registra as 30 decisões aprovadas pelo usuário e incorpora a solicitação posterior de compatibilidade com Voicemeeter Potato e scroll de volume. Escolhas como passos iniciais de scroll, estrutura de crates e exemplos JSON são recomendações de implementação sujeitas a ajuste técnico.
