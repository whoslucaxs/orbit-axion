# Etapa 0 — viabilidade e riscos

Data: 22/11/2026. Escopo: laboratório técnico anterior ao MVP, sem implementar as 31 features do produto.

## Resultado

Há uma base compilável e executável em Rust + Slint, com janelas separadas, adaptador Win32 e dois provedores reais de áudio. O caminho técnico do overlay e da integração Potato mostrou-se viável neste computador. **A Etapa 0 ainda tem validações manuais e de instalação pendentes; não é uma aprovação para release ou para avançar automaticamente às demais etapas.**

Ambiente observado: Windows 11 x64 build 26200; Rust/Cargo 1.98.1 MSVC; Slint/slint-build 1.19.0; escala observada de 100%. Ferramentas C++ do Visual Studio 2022 presentes. Slint usa aqui o backend winit 0.30 com renderer software, fixado pelo Cargo.lock.

## Evidência local

| Prova | Resultado e limite |
| --- | --- |
| Compilação Windows | Check de todos os targets, build dos dois executáveis, fmt e clippy com `-D warnings` aprovados localmente |
| Geometria e wheel | 3 testes aprovados: oito direções/centro/hit regions, deltas e supressão de soltura, descarte de remainder fora do volume |
| Volume e alvos | 2 testes aprovados: saturação/NaN e validação de strips 0..7 e buses 0..4 |
| Hooks | `orbit-probe` instalou e removeu mouse/keyboard hooks para as três configurações, sem armar interceptação; isso não prova supressão real em aplicativos externos |
| Overlay nativo | `--smoke-test` abriu janela real e confirmou NOACTIVATE, TOPMOST, geometria exata e foreground preservado a 100% de escala |
| UI | Laboratório aberto e inspecionado visualmente. O primeiro layout cortava o rodapé; altura mínima ajustada no código. Interações completas ainda requerem o roteiro manual |
| Windows Core Audio | Saída padrão lida com sucesso, sem alterar o nível ou mute |
| Voicemeeter Potato | DLL x64 encontrada no registro; Login, detecção de edição e leitura de Gain/Mute de Strip 0, Strip 7, Bus 0 e Bus 4 bem-sucedidos, com Logout ao liberar |
| Escrita de volume | Implementada pelo worker, mas não exercitada contra os níveis reais do usuário nesta validação |
| Busca sem foco | Hook temporariamente habilitado pelo estado do menu; protótipo limitado a letras ASCII, números, espaço e backspace. Não é a busca F17 pronta |
| Acrílico | DWM aceitou `DWMWA_SYSTEMBACKDROP_TYPE / DWMSBT_TRANSIENTWINDOW`. Aceitação da API não comprova o efeito visual; o anel tem fallback próprio translúcido |
| MSI e rollback | Pesquisa e proposta abaixo; nenhum MSI foi criado ou instalado |

As ferramentas de UI detectaram entrada concorrente do usuário durante a tentativa de interação. A validação automatizada de cliques pelo desktop foi encerrada; não se presume que os controles tenham sido exercitados.

## Descoberta de integração Slint/Win32

Aplicar `WS_EX_NOACTIVATE` uma única vez não funciona: o winit reescreve estilos ao mostrar a janela. O primeiro smoke test falhou nesse requisito. O adaptador agora instala `SetWindowSubclass` **na própria janela do aplicativo**, preserva NOACTIVATE/TOOLWINDOW em `WM_STYLECHANGING`, responde MA_NOACTIVATE a `WM_MOUSEACTIVATE` e remove o subclass no encerramento. Após essa alteração, o teste passou.

Isso não envolve injeção em outro processo. O backend continua responsável pela renderização. `SetWindowPos` usa NOACTIVATE e TOPMOST, mantendo o centro físico solicitado mesmo perto das bordas. DPI misto e recriação de superfícies precisam de testes adicionais. A integração depende da API `unstable-winit-030`, por isso a versão Slint está fixada.

## Entrada e ciclo de vida

- Captura começa desligada e só é armada pelo laboratório; o menu abre exclusivamente pelo botão do mouse escolhido.
- Hooks despacham eventos por canal limitado; callbacks não fazem I/O nem chamadas de áudio. A thread possui message loop Win32.
- A origem do foco é capturada com o acionamento. Troca de foreground fecha o menu; o hook também deixa de interceptar entrada do menu se o foreground não coincide.
- Um heartbeat da UI limita interceptação quando o controlador deixa de responder por 250 ms. Canal cheio desarma a captura e solicita reinício pelo usuário.
- `wheel_adjusted` é marcado até para deltas menores que um notch. A soltura depois de scroll não produz seleção executável.
- Wheel vertical dentro do retângulo do overlay é consumido; apenas a região circular do ícone superior modifica o alvo. Fora desse retângulo passa normalmente. Wheel horizontal ainda não está coberto.
- Botões pressionados consumidos têm as solturas correspondentes consumidas. Fechar o laboratório remove os hooks e encerra o loop.
- Falha de abertura, fullscreen e diferença de integridade ainda não têm a política definitiva de disponibilidade de F28. Não considerar o protótipo apropriado para jogos ou aplicativos elevados.
- Polling de 8 ms é uma decisão temporária do laboratório. Substituir por eventos/wakeups e medir idle antes do MVP residente.

## Áudio

COM e Remote API são criados e usados dentro do worker de áudio; os provedores não são Send/Sync. Cada troca de alvo libera a sessão anterior. Erros invalidam a conexão, e a próxima leitura tenta reconectar. O diagnóstico não inicia o Voicemeeter nem muda seus parâmetros.

O carregamento da DLL usa caminho absoluto da instalação registrada e restringe a busca de dependências ao diretório dessa DLL e ao System32. Nenhuma DLL é copiada. A edição Potato é verificada antes de habilitar os seus alvos. O módulo chama `IsParametersDirty` antes das consultas e relê o valor antes de aplicar deltas, com limites documentados de -60 a +12 dB. Windows limita o volume a 0..100%, preservando o mute.

Pendências: teste real de escrita e restauração de valores; confirmação da latência da Remote API após SetParameterFloat; rajadas/coalescing para não perder ajustes enquanto o servidor processa comandos; labels e seleção completa de alvos; alteração do dispositivo padrão durante uma sessão; volumes de sessão; sliders externos e reconexão após reiniciar Potato. O passo atual é fixo no protótipo, não uma configuração persistida.

## Busca sem ativação

O experimento usa WH_KEYBOARD_LL e só consome os caracteres suportados enquanto o menu está ativo, a opção experimental está ligada e o foreground permanece o original. ESC é tratado independentemente da opção de busca. Teclas e combinações não suportadas passam para o aplicativo original: **não usar essa prova como captura completa de texto**. Não registra o texto pesquisado em arquivo.

Próximo gate: validar layout pt-BR, Shift, AltGr, dead keys, composição Unicode, IME, teclas pressionadas antes da abertura, acessibilidade e atalhos. Só depois integrar fuzzy e navegação dos resultados. Nenhum atalho de teclado abre o menu.

## Decisão preliminar para MSI/rollback

Proposta: WiX Toolset atual para MSI x64 dual-purpose, mantendo um UpgradeCode estável e o mesmo contexto de instalação nas atualizações. Não criar updater antes de comprovar os cenários abaixo numa VM descartável. `dotnet` e `wix` não estavam disponíveis no PATH; a ferramenta não foi instalada no computador do usuário.

1. Provar escopo por usuário e por máquina com propriedades ALLUSERS/MSIINSTALLPERUSER e pastas/redirecionamentos apropriados.
2. Escolher UI MSI que exponha o escopo. Os exemplos de seleção de escopo de Burn usam bootstrapper EXE; não substituem o requisito de distribuir MSI.
3. Provar upgrade com remoção da versão anterior **dentro da transação**, preservando configuração fora dos componentes do programa.
4. Injetar uma falha de instalação em VM e comprovar a restauração da versão anterior. Rollback de uma transação falha não é rollback garantido de uma versão que já instalou e depois falhou em runtime.
5. Provar preservação/remoção de configurações na desinstalação e UAC apenas para os casos necessários.

Antes de fixar a versão da ferramenta, rever os termos e o ciclo de suporte do WiX. A proposta não equivale à aprovação de redistribuição de binários proprietários. Nenhum instalador, autostart, atualização ou mudança do registro foi aplicado.

## Gates restantes da Etapa 0

- Executar [TEST_PLAN.md](TEST_PLAN.md) com mouse físico, dois aplicativos comuns e o Potato.
- Validar foco também em reapertura, clique real no anel e mudança de aplicação.
- Validar 125/150/200%, monitores com DPI misto e bordas.
- Confirmar acrílico visual e decidir material/fallback com o renderer adotado.
- Resolver texto/IME e política para teclas não suportadas antes da busca real.
- Exercitar áudio por scroll e comportamento da Remote API sob eventos rápidos.
- Provar MSI/upgrade/rollback em VM e revisar termos da ferramenta.
- Avaliar GPU, consumo idle e latência antes de transformar o laboratório em aplicação residente.

## Fontes oficiais consultadas

- [Slint: acesso à janela winit](https://docs.slint.dev/latest/docs/rust/slint/winit_030/trait.WinitWindowAccessor)
- [Slint: criação do backend e WindowAttributes](https://docs.slint.dev/latest/docs/rust/slint/struct.BackendSelector)
- [Slint: backends e renderers](https://docs.slint.dev/latest/docs/slint/guide/backends-and-renderers/backends_and_renderers/)
- [Microsoft: SetWindowPos](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos)
- [VB-Audio: SDK/headers oficiais](https://github.com/vburel2018/Voicemeeter-SDK)
- [VB-Audio: Remote API](https://download.vb-audio.com/Download_CABLE/VoicemeeterRemoteAPI.pdf)
- [Microsoft: MSI dual-purpose](https://learn.microsoft.com/en-us/windows/win32/msi/single-package-authoring)
- [WiX: MajorUpgrade e agendamento](https://docs.firegiant.com/wix/schema/wxs/majorupgrade/)
- [WiX: escopo configurável em bundles](https://docs.firegiant.com/wix/whatsnew/configurable_scope_bundles/)
- [WiX: termos da ferramenta](https://docs.firegiant.com/wix/osmf/)
