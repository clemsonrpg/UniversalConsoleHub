# Universal Console Hub (UCH)

O **Universal Console Hub (UCH)** é um projeto criado para trazer de volta a experiência de utilizar um console de verdade no PC.

A ideia é ir além de simplesmente abrir jogos em emuladores. O objetivo é construir um ambiente completo inspirado na experiência original do Xbox 360, com dashboard, Xbox Guide, perfis, conquistas, amigos, mensagens, partidas e outros recursos que faziam parte do console.

O projeto utiliza o Xenia como base para executar jogos de Xbox 360 e desenvolve uma estrutura própria de serviços para conectar todas essas funcionalidades.

## O que já foi desenvolvido

### Experiência Xbox 360
- Integração com o dashboard original do Xbox 360.
- Abertura do Xbox Guide utilizando o botão Guide do controle.
- Navegação pelo Guide com os controles tradicionais do console.
- Integração com o perfil do jogador.
- Leitura de informações e conquistas dos jogos.

### Perfis e recursos sociais
- Sistema próprio de perfis e identidade Xbox.
- Lista de amigos e solicitações de amizade.
- Envio e recebimento de mensagens.
- Status de presença e atividade nos jogos.
- Notificações exibidas dentro da experiência do Xbox 360.
- Sistema próprio de sessões e convites.

### Rede Universal Live
O UCH possui uma infraestrutura própria de serviços, independente dos servidores oficiais da Microsoft.

Entre os recursos desenvolvidos estão:
- Conexão entre diferentes instalações do UCH.
- Descoberta de outros usuários na rede local.
- Pareamento seguro entre computadores.
- Sincronização de amigos, mensagens e presença.
- Sistema de sessões e convites.
- Estrutura para partidas multiplayer pela rede local.

### Jogos e conquistas
- Integração com informações dos jogos executados no Xenia.
- Leitura de dados de perfis e arquivos de conquistas.
- Identificação dos jogos em execução.
- Integração inicial com o sistema de desbloqueio de conquistas.

### Marketplace e conteúdo
- Catálogo próprio de conteúdo.
- Sistema de download com suporte a pausa e retomada.
- Verificação de integridade dos arquivos.
- Instalação de conteúdo com mecanismos de proteção.

## Testes realizados

O projeto já passou por testes automatizados e testes práticos envolvendo diferentes partes do sistema.

Também foram realizados testes com duas instâncias do Hub e duas instâncias do Xenia funcionando simultaneamente no mesmo computador, incluindo comunicação entre usuários, mensagens e notificações.

A infraestrutura de rede e integração com o System Link já possui uma implementação inicial, mas a experiência multiplayer entre dois computadores físicos ainda precisa ser validada.

## Em desenvolvimento

O UCH continua em desenvolvimento. Algumas das próximas etapas são:

- Aperfeiçoar a integração com o Xbox Guide original.
- Corrigir diferenças entre o estado de conexão do Hub e o exibido pelo Guide.
- Validar partidas multiplayer entre computadores diferentes.
- Expandir a integração com os recursos originais do Xbox 360.
- Melhorar a estabilidade e a compatibilidade com jogos.
- Continuar desenvolvendo os recursos sociais e de rede.

## Filosofia do projeto

O objetivo do UCH não é substituir a experiência do Xbox 360 por uma interface moderna de PC.

A proposta é preservar a identidade do console: o dashboard original, o Guide, os controles, os perfis e a maneira como os recursos se conectam dentro do próprio ambiente do Xbox.

O Hub funciona como a infraestrutura que torna tudo isso possível, enquanto o Xenia permite executar os jogos.

**A ideia é fazer o PC se comportar cada vez mais como um Xbox 360 completo, e não apenas como um computador rodando jogos de Xbox 360.**
