# Verificação do ícone do Orbit

`assets/branding/orbit-symbol-icon-large.png` é a arte-fonte quadrada. O
comando `npm run icons:generate` deriva dela os ícones do Tauri, atualiza o
favicon e monta `src-tauri/icons/icon.ico` com as entradas 16, 24, 32, 48, 64,
128 e 256 px. Ele é executado automaticamente pelo `beforeBuildCommand` de
produção.

Após a build, rode `npm.cmd run icons:verify`. Para validar o instalador,
gere `npm.cmd run tauri build -- --bundles nsis`, instale o `*-setup.exe` em
`src-tauri/target/release/bundle/nsis` e, se o Windows mostrar a versão antiga,
encerre o Orbit, limpe o cache de ícones do Explorer e abra-o novamente.
