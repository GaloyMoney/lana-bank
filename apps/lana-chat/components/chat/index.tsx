"use client";

import { useChat } from "ai/react";
import { MessagesSquare } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { useEffect } from "react";
import { ChatInput } from "./input";
import { ChatMessage } from "./message";

const STORAGE_KEY = "chat_messages";

export default function Chat() {
  const {
    messages,
    input,
    handleInputChange,
    handleSubmit,
    isLoading,
    error,
    setMessages,
  } = useChat({
    api: "/api/chat",
    initialMessages: [],
  });

  useEffect(() => {
    const savedMessages = localStorage.getItem(STORAGE_KEY);
    if (savedMessages) {
      setMessages(JSON.parse(savedMessages));
    }
  }, [setMessages]);

  useEffect(() => {
    if (messages.length > 0) {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(messages));
    }
  }, [messages]);

  const handleNewChat = () => {
    localStorage.removeItem(STORAGE_KEY);
    setMessages([]);
  };

  return (
    <div className="flex flex-col h-screen bg-background max-w-5xl mx-auto">
      <NavMenu handleNewChat={handleNewChat} />
      <main className="flex-1 flex flex-col mx-auto w-full relative">
        <div className="flex flex-col h-full mx-2">
          <div className="flex-1 overflow-hidden">
            <ScrollArea className="h-full w-full">
              <div className="py-6">
                {messages.map((message, index) => (
                  <ChatMessage key={index} message={message} />
                ))}
              </div>
            </ScrollArea>
          </div>
          {error && (
            <Alert variant="destructive" className="mx-6 mb-4">
              <AlertDescription>{error.message}</AlertDescription>
            </Alert>
          )}
          <div className="sticky bottom-0">
            <ChatInput
              input={input}
              handleInputChange={handleInputChange}
              handleSubmit={handleSubmit}
              isLoading={isLoading}
            />
          </div>
        </div>
      </main>
    </div>
  );
}

const NavMenu = ({ handleNewChat }: { handleNewChat: () => void }) => {
  return (
    <nav className="flex flex-row justify-between items-center w-full border-b py-4 px-4">
      <div className="text-2xl font-semibold">Chat-bot</div>
      <Button onClick={handleNewChat}>
        <MessagesSquare className="mr-2 h-4 w-4" /> New Chat
      </Button>
    </nav>
  );
};
