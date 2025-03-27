import { Send } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";

type ChatInputProps = {
  input: string;
  handleInputChange: (e: React.ChangeEvent<HTMLTextAreaElement>) => void;
  handleSubmit: (e: React.FormEvent<HTMLFormElement>) => void;
  isLoading: boolean;
};

export const ChatInput = ({
  input,
  handleInputChange,
  handleSubmit,
  isLoading,
}: ChatInputProps) => {
  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      const form = e.currentTarget.form;
      if (form && !isLoading) {
        handleSubmit(
          new Event("submit") as unknown as React.FormEvent<HTMLFormElement>
        );
      }
    }
  };

  return (
    <form
      onSubmit={handleSubmit}
      className="flex gap-2 px-4 mb-2 m-2 py-3 border rounded-md shadow-md bg-background"
    >
      <div className="flex-grow relative">
        <Textarea
          value={input}
          autoFocus
          onChange={handleInputChange}
          onKeyDown={handleKeyDown}
          placeholder="Type your message..."
          className="resize-none min-h-[50px] max-h-[200px] pr-12 focus-visible:ring-0 border-none shadow-none"
          rows={1}
        />
      </div>
      <Button
        type="submit"
        size="icon"
        disabled={isLoading}
        className="h-8 w-8 p-0 self-end mb-1"
      >
        <Send className={`h-4 w-4 ${isLoading ? "animate-pulse" : ""}`} />
      </Button>
    </form>
  );
};
