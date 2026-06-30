import { render, screen } from "@testing-library/react";
import { LoginSection, TermsSection } from "@/src/components/page-sections";

describe("page sections", () => {
  it("新規登録画面に許可ドメイン案内を表示する", () => {
    render(
      <LoginSection
        allowDomain="example.com"
        signInEmail=""
        signInPassword=""
        signUpEmail=""
        signUpPassword=""
        confirmationEmail=""
        confirmationCode=""
        onSignInEmailChange={() => {}}
        onSignInPasswordChange={() => {}}
        onSignUpEmailChange={() => {}}
        onSignUpPasswordChange={() => {}}
        onConfirmationEmailChange={() => {}}
        onConfirmationCodeChange={() => {}}
        onSignIn={() => {}}
        onSignUp={() => {}}
        onConfirm={() => {}}
      />,
    );

    expect(screen.getByText(/example.com/)).toBeInTheDocument();
    expect(screen.getByText(/確認コード入力/)).toBeInTheDocument();
  });

  it("利用規約に必須文言を表示する", () => {
    render(<TermsSection />);

    expect(screen.getByText(/修正も削除もできません/)).toBeInTheDocument();
    expect(screen.getByText(/回復できない可能性があります/)).toBeInTheDocument();
  });
});
