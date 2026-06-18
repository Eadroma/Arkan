import type { ButtonHTMLAttributes, ReactNode } from "react";

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  icon?: ReactNode;
  tone?: "primary" | "quiet";
};

export function Button({
  children,
  className = "",
  icon,
  tone = "primary",
  type = "button",
  ...props
}: ButtonProps): React.JSX.Element {
  return (
    <button className={`button button--${tone} ${className}`.trim()} type={type} {...props}>
      {icon ? <span className="button__icon">{icon}</span> : null}
      <span>{children}</span>
    </button>
  );
}
