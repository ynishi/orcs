import { createTheme, MantineColorsTuple } from '@mantine/core';

// カスタムカラーの定義（必要に応じて追加）
const brandColor: MantineColorsTuple = [
  '#e6f4ff',
  '#bae0ff',
  '#91caff',
  '#69b1ff',
  '#4096ff',
  '#1677ff',
  '#0958d9',
  '#003eb3',
  '#002c8c',
  '#001d66',
];

export const theme = createTheme({
  /** フォント設定 */
  fontFamily: 'Inter, Avenir, Helvetica, Arial, sans-serif',
  fontFamilyMonospace: 'Monaco, Courier, monospace',

  /** 基本フォントサイズ */
  fontSizes: {
    xs: '0.75rem',
    sm: '0.875rem',
    md: '1rem',
    lg: '1.125rem',
    xl: '1.25rem',
  },

  /** ヘッドライン設定 */
  headings: {
    fontFamily: 'Inter, Avenir, Helvetica, Arial, sans-serif',
    fontWeight: '700',
  },

  /** ボーダー半径 */
  radius: {
    xs: '4px',
    sm: '6px',
    md: '8px',
    lg: '12px',
    xl: '16px',
  },

  /** シャドウ設定 */
  shadows: {
    xs: '0 1px 2px rgba(0, 0, 0, 0.1)',
    sm: '0 2px 2px rgba(0, 0, 0, 0.2)',
    md: '0 4px 6px rgba(0, 0, 0, 0.2)',
    lg: '0 8px 16px rgba(0, 0, 0, 0.2)',
    xl: '0 12px 24px rgba(0, 0, 0, 0.3)',
  },

  /** カラーパレット */
  colors: {
    brand: brandColor,
  },

  /** プライマリカラー */
  primaryColor: 'blue',

  /** コンポーネントのデフォルト設定 */
  components: {
    Button: {
      defaultProps: {
        radius: 'md',
      },
      styles: {
        root: {
          fontWeight: 500,
          transition: 'border-color 0.25s',
          '&:hover': {
            borderColor: '#396cd8',
          },
        },
      },
    },
    TextInput: {
      defaultProps: {
        radius: 'md',
      },
      styles: {
        input: {
          fontSize: '1em',
          fontWeight: 500,
          transition: 'border-color 0.25s',
        },
      },
    },
    Container: {
      defaultProps: {
        sizes: {
          xs: '540px',
          sm: '720px',
          md: '960px',
          lg: '1140px',
          xl: '1320px',
        },
      },
    },
  },

  /** その他の設定 */
  other: {
    // カスタム変数を追加可能
  },
});
