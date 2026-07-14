/**
 * ヘッダーの主要表示状態を確認する Storybook ストーリー。
 */
import type { Meta, StoryObj } from '@storybook/nextjs-vite';
import { HeaderView } from '@/components/Header';

const meta = {
  title: 'Components/Header',
  component: HeaderView,
  parameters: {
    layout: 'fullscreen',
  },
  args: {
    title: 'チャット',
    isAuthenticated: false,
    onMenuToggle: () => {},
    onMyPageClick: () => {},
    onLoginClick: () => {},
  },
} satisfies Meta<typeof HeaderView>;

export default meta;
type Story = StoryObj<typeof meta>;

export const NotAuthenticated: Story = {
  name: '未ログイン',
};

export const Authenticated: Story = {
  name: 'ログイン中',
  args: {
    isAuthenticated: true,
  },
};

export const LongTitle: Story = {
  name: 'タイトル長い',
  args: {
    title: 'とても長いページタイトルがここに入るケースのテスト用ストーリーです',
  },
};
