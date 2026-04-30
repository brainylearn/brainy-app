import { useState, useCallback } from "react";
import errorToString from "../utils/errorToString";

export type CallApiFn = <T>(fetch: () => Promise<T>) => Promise<T | undefined>;

interface Props {
	onFinally?: () => void;
}

export default function useApi({ onFinally }: Props = {}) {
	const [isSendingRequest, setIsSendingRequest] = useState<boolean>(false);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);

	const callApi: CallApiFn = useCallback(
		async fetch => {
			setIsSendingRequest(true);

			try {
				return await fetch();
			} catch (e) {
				console.error(e);
				setErrorMessage(errorToString(e));
			} finally {
				setIsSendingRequest(false);
				if (onFinally) onFinally();
			}
		},
		[onFinally],
	);

	const clearErrorMessage = useCallback(() => setErrorMessage(null), []);

	return {
		isSendingRequest,
		errorMessage,
		callApi,
		clearErrorMessage,
	};
}
