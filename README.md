# Image Proc — Processador de Imagens (PDI)

Trabalho 1 de Processamento Digital de Imagens.

## Requisitos implementados

### Básico
- Abrir e salvar imagens nos formatos **BMP, JPG, PNG** (e outros suportados pelo crate `image`)
- Interface gráfica amigável com egui/eframe

### Operações ponto a ponto (cinza e colorido)
- Limiarização
- Negativo
- Brilho
- Correção de gama (potência)
- Transformação linear definida por partes

### Histograma
- Exibição do histograma (canais R, G, B e Intensidade)
- Equalização de histograma em escala de cinza
- Equalização de histograma em imagens coloridas (via espaço **HSI** — equaliza apenas I)

### Filtros espaciais / Convolução
- Filtro genérico por convolução (kernels ímpares 3×3 até 15×15)
- Média (simples)
- Gaussiano (ponderado, com controle de σ)
- Mediana
- Laplaciano (com normalização para visualização)
- Aguçamento por Laplaciano
- High-Boost

### Detecção de bordas
- Sobel X e Sobel Y (normalizados)
- Magnitude do gradiente (não-linear)

### Transformações geométricas
- Escala com interpolação **vizinho mais próximo** e **bilinear**
- Rotação com especificação de pivô e as mesmas interpolações

### Esteganografia
- Ocultação e extração de mensagem por LSB no canal R

### Processamento de cor (requisitos finais)
- Conversão **RGB ↔ HSI**
- Escala de cinza: média aritmética e média ponderada (BT.601)
- Negativo em colorido
- **Chroma-Key** (escolhe cor-chave, distância e imagem de substituição)
- Histograma R, G, B e I
- Equalização de histograma em HSI
- Ajuste de Matiz, Saturação e Intensidade
- Ajuste de canais (R/G/B)
- Sépia
- Todas as operações de cinza também funcionam em imagens coloridas (aplicadas canal a canal ou via HSI quando apropriado)

### Observações sobre DFT
A Transformada Discreta de Fourier (e FFT) **não foi implementada** neste momento (é a parte mais avançada e opcional em alguns critérios). Pode ser adicionada posteriormente em um módulo `fourier.rs`.

## Como compilar e executar

Requer Rust estável recente (recomendado 1.80+):

```bash
cd image_proc   # ou a pasta onde está o Cargo.toml
cargo run --release
```

Dependências principais:
- `eframe` / `egui` — interface gráfica
- `image` — leitura/escrita de arquivos
- `rfd` — diálogos de abrir/salvar

## Estrutura do código (fácil de entender)

```
lib.rs              — reexports dos módulos
image.rs            — GrayImage, RgbImage, conversões RGB↔HSI
point_ops.rs        — operações ponto a ponto + sépia, HSI, chroma-key
histogram.rs        — histograma e equalização (cinza e HSI)
convolution.rs      — convolução genérica + kernels clássicos
filters.rs          — mediana, Sobel, high-boost, aguçamento
geometric.rs        — escala e rotação (nearest / bilinear)
steganography.rs    — LSB
utils.rs            — clamp, normalização, validação de kernel
main.rs             — interface gráfica completa
```

Todos os algoritmos de processamento manipulam diretamente as matrizes de pixels (sem bibliotecas de processamento de imagem prontas, apenas I/O e GUI).
