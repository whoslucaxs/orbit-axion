# Validação 0.2.0

Itens deste roteiro não significam testes aprovados. Execute os comandos do README e registre ambiente e resultado para cada rodada manual.

## Testes automatizados

Core: geometria, wheel parcial, supressão de soltura, configuração inválida, ciclos, perfis e busca. Plataforma: pares de teclas e teclas previamente pressionadas. App: argumentos Windows, atalhos, origem do updater, persistência/recuperação e pacotes com ícones. Áudio: limites e alvos.

Smoke: overlay real sem mudança de foreground, estilos NOACTIVATE/TOOLWINDOW e centro exato. UI: valores carregados preservados ao montar preferências. Probe: hooks desarmados e leituras reais de Core Audio/Potato.

## Rodada manual em desktop de teste

1. Testar Meio/Mouse 4/Mouse 5 em clique e hold; toque curto, segurar, centro, oito direções, ESC, clique fora e troca de foreground. Pausar e sair devem devolver a entrada original.
2. Testar submenu, voltar, busca fuzzy, acentos, dead keys e teclas já pressionadas. IME deve orientar seleção pelo mouse. Overlay não pode receber foco.
3. Criar perfil por EXE, prioridade, herança, modo manual e desativação por aplicativo. Reiniciar e confirmar persistência. Reordenar posições, menus e passos da macro.
4. Em editor de texto descartável, testar atalhos/texto Unicode/macros e cancelamento. Testar lançamento com espaços, argumentos, scripts e UAC em VM. Confirmar que nada executa antes de o anel fechar.
5. Salvar níveis/mute originais. Testar Windows padrão/endpoint, Potato Strip 0/7 e Bus 0/4, limites, passos, labels, coalescing, mudança externa e reconexão. Restaurar níveis. Rolar fora do ícone não muda áudio; soltar após wheel não dispara ação.
6. Importar/exportar pacote completo e perfil com ícones em outra pasta. Confirmar que Global é preservado na importação individual. Corromper somente configuração de teste e verificar backup/quarentena.
7. Testar claro/escuro/sistema, tamanhos, opacidade, sons, idioma e DPI misto, inclusive bordas. Medir uso ocioso e sessões prolongadas.
8. Em VM limpa: MSI por usuário e máquina, UAC, upgrade, cancelamento, rollback por falha, desinstalação preservando/removendo dados. Não usar dados pessoais nessa rodada.
9. Em release de teste: ausência de rede, repositório privado, digest ausente/incorreto, download interrompido, recusa e instalação autorizada. Nunca aceitar binário sem digest válido.
10. Testar inicialização, instância única, watchdog, falhas repetidas, modo seguro e saída pela bandeja. Confirmar ausência de hooks após encerrar.

Jogos, processos elevados e leitores de tela precisam de ambientes específicos. Não declarar compatibilidade sem essa rodada.
