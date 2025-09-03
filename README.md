**Como rodar o teste**

**Instalar Rust**
Acesse o link a baixo e faça a instalação recomendada, se o sistema for 64bits instale a versão correta para ele.
[Rust Lang](https://www.rust-lang.org/tools/install)

**Instale as dependências do Rust para Windows 32bits (Única Plataforma atualmente):**
    `rustup target add i686-pc-windows-msvc`

**Instale a extensão do Rust no VsCode**
[rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

**Rodar o teste**
`cargo run --target i686-pc-windows-msvc` 
Se rodar sem o target vai dar erro!!

**acbr_lib:** Todo o nosso codigo que mapea a Lib e as funções, um pacote para ser utilizado em multiplos projetos

**src:** O código de teste.

**Info da Lib**
Versão: 1.0.2.174
MultiThread
StdCall
32Bits

**Info PC de teste**
Edição	Windows Server 2022 Datacenter
Versão	21H2
Compilação do SO 20348.2762
