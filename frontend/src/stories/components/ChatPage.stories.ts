import type { Meta, StoryObj } from '@storybook/nextjs-vite';
import { ChatPageView } from '@/components/pages/ChatPage';

const sampleMessages = [
  {
    user_name: 'yamada',
    body: 'おはようございます！',
    created_at: '2025-01-15T09:00:00Z',
    is_from_user: false,
  },
  {
    user_name: 'myself',
    body: '今日もよろしくお願いします。',
    created_at: '2025-01-15T09:01:00Z',
    is_from_user: true,
  },
  {
    user_name: 'suzuki',
    body: '本日の会議は14:00からです。よろしくお願いします。',
    created_at: '2025-01-15T09:05:00Z',
    is_from_user: false,
  },
];

const meta = {
  title: 'Pages/ChatPage',
  component: ChatPageView,
  parameters: {
    layout: 'fullscreen',
  },
  args: {
    isAuthenticated: true,
    isAuthChecking: false,
    isLoading: false,
    isPosting: false,
    messages: sampleMessages,
    draft: '',
    hasMore: false,
    onChangeDraft: () => {},
    onRefresh: () => {},
    onSubmit: () => {},
    onLoadMore: () => {},
    onNavigate: () => {},
  },
} satisfies Meta<typeof ChatPageView>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  name: 'タイムライン表示',
};

export const Loading: Story = {
  name: '読み込み中',
  args: {
    isLoading: true,
    messages: [],
  },
};

export const Empty: Story = {
  name: '投稿なし',
  args: {
    messages: [],
  },
};

export const Posting: Story = {
  name: '投稿中',
  args: {
    isPosting: true,
    draft: 'テスト投稿です',
  },
};

export const NotAuthenticated: Story = {
  name: '未ログイン',
  args: {
    isAuthenticated: false,
    messages: [],
  },
};
