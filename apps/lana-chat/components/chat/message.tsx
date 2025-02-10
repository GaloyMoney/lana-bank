"use client";

import { Message } from "ai/react";
import { User } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeHighlight from "rehype-highlight";
import { components } from "./markdown";

export const ChatMessage = ({ message }: { message: Message }) => {
  const isUser = message.role === "user";
  const content = typeof message.content === "string" ? message.content : "";
  if (content.trim() === "") return null;

  if (isUser) {
    return (
      <div className="flex items-end justify-end mb-6 gap-3 text-sm">
        <Card className="bg-primary text-primary-foreground">
          <CardContent className="py-2 px-4">
            <p className="whitespace-pre-wrap">{content}</p>
          </CardContent>
        </Card>
        <Avatar>
          <AvatarFallback>
            <User />
          </AvatarFallback>
        </Avatar>
      </div>
    );
  }

  return (
    <div className="flex items-start mb-6 gap-3 text-sm">
      <Avatar>
        <AvatarFallback>AI</AvatarFallback>
      </Avatar>
      <Card className="bg-secondary text-secondary-foreground">
        <CardContent className="py-2 px-4">
          {content && (
            <ReactMarkdown
              remarkPlugins={[remarkGfm]}
              rehypePlugins={[rehypeHighlight]}
              components={components}
            >
              {content}
            </ReactMarkdown>
          )}
        </CardContent>
      </Card>
    </div>
  );
};
