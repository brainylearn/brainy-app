import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import { useEffect } from "react";

interface IProps {
    autofocus: boolean;
}

export default function AutoFocusPlugin({ autofocus }: IProps) {
    const [editor] = useLexicalComposerContext();

    useEffect(() => {
        if (autofocus)
        editor.focus();
    }, [editor, autofocus]);

    return null;
}
