# EmuChan 🕹️

Este é um **projeto de hobby** para aprender Rust e explorar a emulação de hardware. A ideia é emular o GameBoy original, não com a intenção de criar um produto final, mas como uma forma de aprendizado e para passar o tempo de maneira divertida.

> **Nota:** Este projeto não busca ser uma ferramenta de produção. O foco está no processo de aprendizagem e na exploração da emulação de hardware de forma prática.

## 🚀 Recursos

✅ Emulação da CPU Z80-like do GameBoy  
✅ Implementação do barramento de memória e mapeamento de registradores  
✅ Renderização via PPU (em desenvolvimento)  
✅ Suporte ao boot ROM do GameBoy (em desenvolvimento)  
✅ Compatibilidade com jogos e testes de hardware (em desenvolvimento)

## 📚 Referências

Esses materiais foram essenciais para a construção do emulador:

- 📖 [PanDocs - Especificações do GameBoy](https://gbdev.io/pandocs/Specifications.html)
- 🛠️ [Mooneye - Testes e ferramentas de depuração](https://github.com/Gekkio/mooneye-gb)
- 🔢 [Tabela de OpCodes do GameBoy](https://izik1.github.io/gbops/index.html)
- 🏁 [Disassembly da Boot ROM do GameBoy](https://gist.github.com/drhelius/6063288)
- 🏁 [sm83 - GameBoy cpu json tests](https://github.com/SingleStepTests/sm83)

## 💻 Como Rodar

1️⃣ Clone o repositório:

```sh
git clone https://github.com/joserochadev/emuchan.git
cd emuchan
```

2️⃣ Compile o projeto:

```sh
cargo build --release
```

3️⃣ Execute o emulador:

```sh
cargo run --release
```

## 📌 Contribuições

Este projeto não tem fins comerciais e é voltado para o aprendizado. No entanto, se você tem sugestões ou melhorias, sinta-se à vontade para abrir uma issue ou enviar um pull request!
