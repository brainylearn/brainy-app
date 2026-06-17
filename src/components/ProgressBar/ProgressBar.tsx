import styles from "./styles.module.css";

interface Props {
	value: number;
}

export default function ProgressBar({ value }: Props) {
	return (
		<div className={styles["progress-track"]}>
			<div
				className={styles["progress-fill"]}
				style={{ width: `${value}%` }}
			/>
		</div>
	);
}
