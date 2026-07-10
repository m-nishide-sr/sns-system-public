import type { Meta, StoryObj } from '@storybook/nextjs-vite';
import { ToastCard } from '@/components/Toast';

const meta = {
  title: 'Components/Toast',
  component: ToastCard,
  parameters: {
    layout: 'centered',
  },
  args: {
    onClose: () => {},
  },
} satisfies Meta<typeof ToastCard>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Success: Story = {
  name: '成功トースト',
  args: {
    toast: {
      id: '1',
      tone: 'success',
      message: '投稿しました。',
    },
  },
};

export const Error: Story = {
  name: 'エラートースト',
  args: {
    toast: {
      id: '2',
      tone: 'error',
      message: '投稿に失敗しました。もう一度お試しください。',
    },
  },
};
