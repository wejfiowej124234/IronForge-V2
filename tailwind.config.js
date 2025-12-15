/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: 'class',
  content: [
    "./src/**/*.rs",
    "./index.html",
    "./src/**/*.html",
    "./src/**/*.css",
  ],
  theme: {
    extend: {
      // Zodia Wallet Design System - Enterprise Dark × Apple Minimal × Web3 Tech
      colors: {
        // Background Layers
        obsidian: '#0B0B0F',    // 主背景 - 深黑曜石
        void: '#111113',        // 次级背景 - 虚空黑
        slate: '#141417',       // 三级背景 - 板岩灰
        charcoal: '#1A1A1F',    // 卡片背景 - 炭灰
        
        // Primary Colors (Brand)
        'iron-purple': '#6C4BFF',  // 品牌紫
        'iron-blue': '#4C6FFF',    // 品牌蓝
        'iron-violet': '#875CFF',  // 渐变紫
        
        // Accent Colors (Tech)
        'neon-cyan': '#00E0FF',    // 霓虹青
        'accent-teal': '#23C6D9',  // 青绿
        
        // Status Colors
        success: {
          DEFAULT: '#22C55E',
          emerald: '#10B981',
        },
        warning: {
          DEFAULT: '#FACC15',
          amber: '#F59E0B',
        },
        danger: {
          DEFAULT: '#EF4444',
          crimson: '#DC2626',
        },
        info: {
          DEFAULT: '#3B82F6',
        },
        
        // Chain Identity Colors
        chain: {
          btc: '#F7931A',      // Bitcoin
          eth: '#627EEA',      // Ethereum
          sol: '#14F195',      // Solana
          ton: '#0098EA',      // TON
          bsc: '#F3BA2F',      // BSC
          polygon: '#8247E5',  // Polygon
        },
        
        // Text Colors (Alpha variations)
        text: {
          primary: 'rgba(255, 255, 255, 0.95)',
          secondary: 'rgba(255, 255, 255, 0.65)',
          tertiary: 'rgba(255, 255, 255, 0.45)',
          disabled: 'rgba(255, 255, 255, 0.25)',
        },
      },
      
      // Typography System
      fontFamily: {
        display: ['Inter', '-apple-system', 'BlinkMacSystemFont', 'sans-serif'],
        sans: ['Inter', '-apple-system', 'BlinkMacSystemFont', 'sans-serif'],
        mono: ['JetBrains Mono', 'SF Mono', 'Fira Code', 'monospace'],
      },
      
      fontSize: {
        // Display - Hero 标题
        '5xl': ['3rem', { lineHeight: '1.1', letterSpacing: '-0.02em', fontWeight: '700' }],
        '4xl': ['2.25rem', { lineHeight: '1.2', letterSpacing: '-0.02em', fontWeight: '700' }],
        // Heading - 章节标题
        '3xl': ['1.875rem', { lineHeight: '1.25', letterSpacing: '-0.01em', fontWeight: '600' }],
        '2xl': ['1.5rem', { lineHeight: '1.3', letterSpacing: '-0.01em', fontWeight: '600' }],
        'xl': ['1.25rem', { lineHeight: '1.4', fontWeight: '600' }],
        // Body - 正文
        'lg': ['1.125rem', { lineHeight: '1.5', fontWeight: '400' }],
        'base': ['1rem', { lineHeight: '1.5', fontWeight: '400' }],
        'sm': ['0.875rem', { lineHeight: '1.5', fontWeight: '400' }],
        'xs': ['0.75rem', { lineHeight: '1.5', fontWeight: '400' }],
      },
      
      fontWeight: {
        bold: '700',
        semibold: '600',
        medium: '500',
        regular: '400',
        light: '300',
      },
      
      // Border Radius (Apple-style)
      borderRadius: {
        'sm': '8px',
        'DEFAULT': '12px',
        'md': '12px',
        'lg': '16px',
        'xl': '20px',
        '2xl': '24px',
        'apple': '20px',      // Apple 标准圆角
        'apple-lg': '22px',   // Apple 强调圆角
        'full': '9999px',
      },
      
      // Gradient Backgrounds
      backgroundImage: {
        // Primary Gradients
        'gradient-primary': 'linear-gradient(135deg, #4C6FFF 0%, #875CFF 100%)',
        'gradient-purple': 'linear-gradient(135deg, #6C4BFF 0%, #875CFF 100%)',
        'gradient-cyan': 'linear-gradient(135deg, #23C6D9 0%, #00E0FF 100%)',
        
        // Glass Effect
        'glass': 'linear-gradient(180deg, rgba(255, 255, 255, 0.05) 0%, rgba(255, 255, 255, 0.02) 100%)',
        'glass-elevated': 'linear-gradient(180deg, rgba(255, 255, 255, 0.08) 0%, rgba(255, 255, 255, 0.04) 100%)',
        
        // Status Gradients
        'gradient-success': 'linear-gradient(135deg, #22C55E 0%, #10B981 100%)',
        'gradient-danger': 'linear-gradient(135deg, #EF4444 0%, #DC2626 100%)',
        'gradient-warning': 'linear-gradient(135deg, #FACC15 0%, #F59E0B 100%)',
        
        // Tag Gradients
        'gradient-hot': 'linear-gradient(135deg, #EF4444 0%, #F59E0B 100%)',
        'gradient-new': 'linear-gradient(135deg, #4C6FFF 0%, #875CFF 100%)',
        
        // Tech Background
        'tech-grid': 'radial-gradient(circle at 25% 25%, rgba(108, 75, 255, 0.05) 0%, transparent 50%), radial-gradient(circle at 75% 75%, rgba(0, 224, 255, 0.05) 0%, transparent 50%)',
      },
      
      // Backdrop Blur (Glass Effect)
      backdropBlur: {
        'glass': '16px',
        'glass-light': '12px',
        'glass-heavy': '24px',
      },
      
      // Box Shadow (Elevation)
      boxShadow: {
        'card': '0 8px 32px rgba(0, 0, 0, 0.12)',
        'card-elevated': '0 12px 48px rgba(108, 75, 255, 0.15)',
        'card-hover': '0 16px 64px rgba(108, 75, 255, 0.2)',
        'button': '0 4px 16px rgba(108, 75, 255, 0.3)',
        'button-hover': '0 8px 24px rgba(108, 75, 255, 0.4)',
        'glow-purple': '0 0 20px rgba(108, 75, 255, 0.3), 0 16px 64px rgba(108, 75, 255, 0.2)',
        'glow-cyan': '0 0 20px rgba(0, 224, 255, 0.3), 0 16px 64px rgba(0, 224, 255, 0.2)',
      },
      
      // Spacing Scale
      spacing: {
        '18': '4.5rem',   // 72px
        '88': '22rem',    // 352px
        '128': '32rem',   // 512px
      },
      
      // Animation
      animation: {
        'shimmer': 'shimmer 1.5s ease-in-out infinite',
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'pulse-shimmer': 'pulse-shimmer 2s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'float': 'float 6s ease-in-out infinite',
        'glow': 'glow 2s ease-in-out infinite',
        'spin-slow': 'spin 3s linear infinite',
        'in': 'in 0.2s ease-out',
        'fade-in': 'fade-in 0.2s ease-out',
        'zoom-in-95': 'zoom-in-95 0.3s ease-out',
        'slide-in-from-right-5': 'slide-in-from-right-5 0.3s ease-out',
        'slide-in-from-top-2': 'slide-in-from-top-2 0.3s ease-out',
      },
      
      keyframes: {
        shimmer: {
          '0%': { backgroundPosition: '-200% 0' },
          '100%': { backgroundPosition: '200% 0' },
        },
        'pulse-shimmer': {
          '0%, 100%': { 
            opacity: '0.4',
            backgroundPosition: '-200% 0',
          },
          '50%': { 
            opacity: '0.8',
            backgroundPosition: '200% 0',
          },
        },
        float: {
          '0%, 100%': { transform: 'translateY(0px)' },
          '50%': { transform: 'translateY(-10px)' },
        },
        glow: {
          '0%, 100%': { opacity: '0.5' },
          '50%': { opacity: '1' },
        },
        'in': {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        'fade-in': {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        'zoom-in-95': {
          '0%': { opacity: '0', transform: 'scale(0.95)' },
          '100%': { opacity: '1', transform: 'scale(1)' },
        },
        'slide-in-from-right-5': {
          '0%': { opacity: '0', transform: 'translateX(20px)' },
          '100%': { opacity: '1', transform: 'translateX(0)' },
        },
        'slide-in-from-top-2': {
          '0%': { opacity: '0', transform: 'translateY(-8px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
      },
      
      // Transition
      transitionTimingFunction: {
        'apple': 'cubic-bezier(0.4, 0, 0.2, 1)',
        'smooth': 'cubic-bezier(0.4, 0, 0.2, 1)',
      },
      
      // Z-Index Scale
      zIndex: {
        'modal': '100',
        'toast': '110',
        'overlay': '90',
        'dropdown': '80',
        'sticky': '50',
        'fixed': '40',
        'base': '1',
      },
    },
  },
  plugins: [
    // Custom Utilities
    function({ addUtilities }) {
      const newUtilities = {
        // Glass Card
        '.glass-card': {
          background: 'rgba(255, 255, 255, 0.04)',
          backdropFilter: 'blur(16px)',
          border: '1px solid rgba(255, 255, 255, 0.06)',
        },
        '.glass-card-elevated': {
          background: 'rgba(255, 255, 255, 0.06)',
          backdropFilter: 'blur(18px)',
          border: '1px solid rgba(108, 75, 255, 0.2)',
        },
        
        // Text Gradient
        '.text-gradient': {
          background: 'linear-gradient(135deg, #FFFFFF, #B0B0FF)',
          '-webkit-background-clip': 'text',
          '-webkit-text-fill-color': 'transparent',
          'background-clip': 'text',
        },
        
        // Hardware Acceleration
        '.gpu-accelerated': {
          transform: 'translateZ(0)',
          willChange: 'transform',
        },
      }
      addUtilities(newUtilities, ['responsive', 'hover'])
    },
  ],
}
