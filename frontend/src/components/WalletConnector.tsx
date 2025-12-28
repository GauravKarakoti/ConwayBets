import { DynamicWidget } from "@dynamic-labs/sdk-react-core";

export function WalletConnector() {
    return (
        <DynamicWidget
            variant="modal"
            innerButtonComponent={
                <button className="connect-wallet-btn">
                    Connect Wallet
                </button>
            }
        />
    );
}