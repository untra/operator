import type { AnchorHTMLAttributes } from "react";
import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";
import styles from "./MarkdownView.module.css";

export type MarkdownViewProps = {
  source: string;
  className?: string;
};

const REMARK_PLUGINS = [remarkGfm];

function ExternalLink({ children, ...props }: AnchorHTMLAttributes<HTMLAnchorElement>) {
  return (
    <a {...props} target="_blank" rel="noopener noreferrer">
      {children}
    </a>
  );
}

const COMPONENTS = { a: ExternalLink };

/** Markdown with GitHub extensions; raw HTML is never rendered. */
export function MarkdownView({ source, className }: MarkdownViewProps) {
  return (
    <div className={`${styles.markdown} ${className ?? ""}`}>
      <Markdown remarkPlugins={REMARK_PLUGINS} components={COMPONENTS}>
        {source}
      </Markdown>
    </div>
  );
}
