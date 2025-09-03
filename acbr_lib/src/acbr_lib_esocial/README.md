[Link para a documentação do ACBrLibEsocial](https://acbr.sourceforge.io/ACBrLib/ACBrLibeSocial.html)

Algumas funções dessa DLL pede para ser passado uma referencia de variável e o tamanho, para que ela seja preenchida.
Para evitar possíveis problemas de ponteiro de memoria, fiz com que o Rust seja responsável por essa variável e seu retorno.
