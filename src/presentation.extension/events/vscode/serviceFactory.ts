import { IServiceCollection } from '#domain/di';
import { IDomainServices } from '#domain/services';
import { nameOf } from '#domain/utils';
import {
  IExtensionServices,
  OnActiveTextEditorChange,
  OnTextDocumentChange,
  OnTextDocumentClose,
  OnTextDocumentSave
} from 'presentation.extension';

export function addOnActiveTextEditorChange(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onActiveTextEditorChange;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) =>
      new OnActiveTextEditorChange(
        container.extension.state,
        container.GetSuggestionProvider,
        container.logger.child({ logGroup: serviceName })
      ),
    true
  )
}

export function addOnTextDocumentChange(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onTextDocumentChange;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) =>
      new OnTextDocumentChange(
        container.GetSuggestionProvider,
        container.versionLensState,
        container.logger.child({ logGroup: serviceName })
      ),
    true
  )
}

export function addOnTextDocumentClose(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onTextDocumentClose;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) =>
      new OnTextDocumentClose(
        container.GetSuggestionProvider,
        container.logger.child({ logGroup: serviceName })
      ),
    true
  )
}

export function addOnTextDocumentSave(services: IServiceCollection) {
  const serviceName = nameOf<IExtensionServices>().onTextDocumentSave;
  services.addSingleton(
    serviceName,
    (container: IDomainServices & IExtensionServices) =>
      new OnTextDocumentSave(
        container.GetSuggestionProvider,
        container.extension.state,
        container.logger.child({ logGroup: serviceName })
      ),
    true
  )
}